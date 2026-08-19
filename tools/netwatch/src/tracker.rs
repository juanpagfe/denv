//! Connection tracker: combines /proc/net sockets with /proc/{pid} process info
//! into a unified view. Supports incremental updates.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use crate::config::Config;
use crate::dns::DnsCache;
use crate::proc_net::{self, Protocol, TcpState};
use crate::proc_pid;

/// A tracked connection with process info and metadata.
#[derive(Debug, Clone)]
pub struct Connection {
    pub local: SocketAddr,
    pub remote: SocketAddr,
    pub protocol: Protocol,
    pub state: TcpState,
    pub pid: Option<u32>,
    pub process_name: String,
    pub user: String,
    pub hostname: Option<String>,
    pub first_seen: Instant,
}

/// Connection key for deduplication.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ConnKey {
    local: SocketAddr,
    remote: SocketAddr,
    protocol: Protocol,
}

pub struct Tracker {
    connections: HashMap<ConnKey, Connection>,
    dns: DnsCache,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            dns: DnsCache::new(),
        }
    }

    /// Refresh the connection list from /proc.
    /// Returns the current snapshot of connections.
    pub fn refresh(&mut self, config: &Config) -> Vec<Connection> {
        let sockets = proc_net::read_all_sockets();
        let inode_map = proc_pid::build_inode_map();
        let now = Instant::now();

        // Build new connection set
        let mut new_conns: HashMap<ConnKey, Connection> = HashMap::new();

        for socket in &sockets {
            // Apply protocol filter
            if let Some(ref proto) = config.filter_protocol {
                let proto_lower = proto.to_lowercase();
                match socket.protocol {
                    Protocol::Tcp if proto_lower != "tcp" => continue,
                    Protocol::Udp if proto_lower != "udp" => continue,
                    _ => {}
                }
            }

            // Apply state filters
            if config.only_established && !socket.state.is_established() {
                continue;
            }
            if config.only_listening && !socket.state.is_listening() {
                continue;
            }

            let key = ConnKey {
                local: socket.local,
                remote: socket.remote,
                protocol: socket.protocol,
            };

            let proc_info = inode_map.get(&socket.inode);

            let (pid, process_name) = match proc_info {
                Some(info) => (Some(info.pid), info.name.clone()),
                None => (None, "-".to_string()),
            };

            let uid = proc_info.map(|i| i.uid).unwrap_or(socket.uid);
            let user = proc_pid::uid_to_name(uid);

            // Apply filters
            if let Some(filter_pid) = config.filter_pid {
                if pid != Some(filter_pid) {
                    continue;
                }
            }

            if let Some(ref filter_process) = config.filter_process {
                if !process_name.to_lowercase().contains(&filter_process.to_lowercase()) {
                    continue;
                }
            }

            if let Some(ref filter_user) = config.filter_user {
                if !user.to_lowercase().contains(&filter_user.to_lowercase()) {
                    continue;
                }
            }

            if let Some(filter_port) = config.filter_port {
                if socket.local.port() != filter_port && socket.remote.port() != filter_port {
                    continue;
                }
            }

            if let Some(ref filter_host) = config.filter_host {
                let matches = socket.remote.ip().to_string().contains(filter_host)
                    || socket.local.ip().to_string().contains(filter_host);
                let hostname_match = if config.resolve_dns {
                    self.dns
                        .lookup(socket.remote.ip())
                        .map(|h| h.to_lowercase().contains(&filter_host.to_lowercase()))
                        .unwrap_or(false)
                } else {
                    false
                };
                if !matches && !hostname_match {
                    continue;
                }
            }

            // DNS resolution
            let hostname = if config.resolve_dns {
                self.dns.lookup(socket.remote.ip())
            } else {
                None
            };

            // Preserve first_seen from previous refresh
            let first_seen = self
                .connections
                .get(&key)
                .map(|c| c.first_seen)
                .unwrap_or(now);

            new_conns.insert(
                key,
                Connection {
                    local: socket.local,
                    remote: socket.remote,
                    protocol: socket.protocol,
                    state: socket.state,
                    pid,
                    process_name,
                    user,
                    hostname,
                    first_seen,
                },
            );
        }

        self.connections = new_conns;

        let mut result: Vec<Connection> = self.connections.values().cloned().collect();

        // Sort: established first, then by process name
        result.sort_by(|a, b| {
            b.state
                .is_established()
                .cmp(&a.state.is_established())
                .then_with(|| a.process_name.cmp(&b.process_name))
                .then_with(|| a.local.cmp(&b.local))
        });

        // Limit
        result.truncate(config.max_connections);

        result
    }
}
