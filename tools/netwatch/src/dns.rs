//! Non-blocking DNS resolution cache.
//! Resolves IPs to hostnames in the background to avoid blocking the UI.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread;

/// A DNS cache that resolves addresses in the background.
pub struct DnsCache {
    cache: Arc<Mutex<HashMap<IpAddr, DnsEntry>>>,
}

enum DnsEntry {
    Resolving,
    Resolved(String),
    Failed,
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Look up an IP address. Returns the hostname if resolved,
    /// or None if still resolving or failed. Triggers background
    /// resolution on first lookup.
    pub fn lookup(&self, ip: IpAddr) -> Option<String> {
        // Skip loopback and unspecified
        if ip.is_loopback() || ip.is_unspecified() {
            return None;
        }

        let mut cache = self.cache.lock().unwrap();

        match cache.get(&ip) {
            Some(DnsEntry::Resolved(name)) => Some(name.clone()),
            Some(DnsEntry::Resolving) | Some(DnsEntry::Failed) => None,
            None => {
                // Start background resolution
                cache.insert(ip, DnsEntry::Resolving);
                let cache_ref = self.cache.clone();
                thread::spawn(move || {
                    let result = reverse_lookup(ip);
                    let mut cache = cache_ref.lock().unwrap();
                    match result {
                        Some(name) => {
                            cache.insert(ip, DnsEntry::Resolved(name));
                        }
                        None => {
                            cache.insert(ip, DnsEntry::Failed);
                        }
                    }
                });
                None
            }
        }
    }
}

fn reverse_lookup(ip: IpAddr) -> Option<String> {
    use dns_lookup::lookup_addr;
    lookup_addr(&ip).ok().filter(|name| {
        // Only return if it actually resolved to a name, not just the IP back
        name.parse::<IpAddr>().is_err()
    })
}
