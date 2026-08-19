//! Parse /proc/net/tcp, /proc/net/udp, /proc/net/tcp6, /proc/net/udp6
//! to get socket information without depending on external tools.

use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Raw socket entry parsed from /proc/net/*.
#[derive(Debug, Clone)]
pub struct SocketEntry {
    pub local: SocketAddr,
    pub remote: SocketAddr,
    pub state: TcpState,
    pub inode: u64,
    pub uid: u32,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

/// TCP connection states from the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown(u8),
}

impl TcpState {
    fn from_hex(val: u8) -> Self {
        match val {
            0x01 => Self::Established,
            0x02 => Self::SynSent,
            0x03 => Self::SynRecv,
            0x04 => Self::FinWait1,
            0x05 => Self::FinWait2,
            0x06 => Self::TimeWait,
            0x07 => Self::Close,
            0x08 => Self::CloseWait,
            0x09 => Self::LastAck,
            0x0A => Self::Listen,
            0x0B => Self::Closing,
            other => Self::Unknown(other),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Established => "ESTABLISHED",
            Self::SynSent => "SYN_SENT",
            Self::SynRecv => "SYN_RECV",
            Self::FinWait1 => "FIN_WAIT1",
            Self::FinWait2 => "FIN_WAIT2",
            Self::TimeWait => "TIME_WAIT",
            Self::Close => "CLOSE",
            Self::CloseWait => "CLOSE_WAIT",
            Self::LastAck => "LAST_ACK",
            Self::Listen => "LISTEN",
            Self::Closing => "CLOSING",
            Self::Unknown(_) => "UNKNOWN",
        }
    }

    pub fn is_established(&self) -> bool {
        matches!(self, Self::Established)
    }

    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listen)
    }
}

/// Read all sockets from /proc/net (TCP + UDP, IPv4 + IPv6).
pub fn read_all_sockets() -> Vec<SocketEntry> {
    let mut sockets = Vec::new();
    read_proc_net("/proc/net/tcp", Protocol::Tcp, false, &mut sockets);
    read_proc_net("/proc/net/udp", Protocol::Udp, false, &mut sockets);
    read_proc_net("/proc/net/tcp6", Protocol::Tcp, true, &mut sockets);
    read_proc_net("/proc/net/udp6", Protocol::Udp, true, &mut sockets);
    sockets
}

fn read_proc_net(path: &str, protocol: Protocol, ipv6: bool, out: &mut Vec<SocketEntry>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // File may not exist or not be readable
    };

    for line in content.lines().skip(1) {
        if let Some(entry) = parse_socket_line(line, protocol, ipv6) {
            out.push(entry);
        }
    }
}

/// Parse a single line from /proc/net/tcp or similar.
///
/// Format:
///   sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
///    0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345
fn parse_socket_line(line: &str, protocol: Protocol, ipv6: bool) -> Option<SocketEntry> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }

    let local = parse_addr(fields[1], ipv6)?;
    let remote = parse_addr(fields[2], ipv6)?;
    let state_hex = u8::from_str_radix(fields[3], 16).ok()?;
    let uid = fields[7].parse::<u32>().ok()?;
    let inode = fields[9].parse::<u64>().ok()?;

    Some(SocketEntry {
        local,
        remote,
        state: TcpState::from_hex(state_hex),
        inode,
        uid,
        protocol,
    })
}

/// Parse an address field like "0100007F:0035" into a SocketAddr.
fn parse_addr(s: &str, ipv6: bool) -> Option<SocketAddr> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let port = u16::from_str_radix(parts[1], 16).ok()?;

    let ip: IpAddr = if ipv6 {
        parse_ipv6_hex(parts[0])?.into()
    } else {
        parse_ipv4_hex(parts[0])?.into()
    };

    Some(SocketAddr::new(ip, port))
}

/// Parse a hex-encoded IPv4 address (stored in little-endian).
fn parse_ipv4_hex(s: &str) -> Option<Ipv4Addr> {
    let val = u32::from_str_radix(s, 16).ok()?;
    Some(Ipv4Addr::from(val.to_be()))
}

/// Parse a hex-encoded IPv6 address from /proc/net/tcp6.
/// The kernel stores each 32-bit group in host byte order.
fn parse_ipv6_hex(s: &str) -> Option<Ipv6Addr> {
    if s.len() != 32 {
        return None;
    }

    let mut octets = [0u8; 16];
    for i in 0..4 {
        let group = &s[i * 8..(i + 1) * 8];
        let val = u32::from_str_radix(group, 16).ok()?;
        let bytes = val.to_be_bytes();
        // Kernel stores groups in little-endian order within each 4-byte word
        octets[i * 4] = bytes[3];
        octets[i * 4 + 1] = bytes[2];
        octets[i * 4 + 2] = bytes[1];
        octets[i * 4 + 3] = bytes[0];
    }

    Some(Ipv6Addr::from(octets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4() {
        // 0100007F = 127.0.0.1 in little-endian
        let ip = parse_ipv4_hex("0100007F").unwrap();
        assert_eq!(ip, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn test_parse_addr() {
        let addr = parse_addr("0100007F:0035", false).unwrap();
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(addr.port(), 53);
    }
}
