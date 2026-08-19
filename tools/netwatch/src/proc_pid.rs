//! Map socket inodes to PIDs by scanning /proc/{pid}/fd symlinks.
//! This is how the kernel exposes process-to-socket relationships.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Information about a process owning a socket.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub uid: u32,
}

/// Build a map from socket inode → ProcessInfo by scanning /proc.
/// Requires appropriate permissions (root or same user).
pub fn build_inode_map() -> HashMap<u64, ProcessInfo> {
    let mut map = HashMap::new();

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return map,
    };

    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Only look at numeric directories (PIDs)
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let pid_path = entry.path();

        // Read process name from /proc/{pid}/comm
        let proc_name = read_comm(&pid_path);

        // Read UID from /proc/{pid}/status
        let uid = read_uid(&pid_path);

        // Scan /proc/{pid}/fd for socket inodes
        let fd_path = pid_path.join("fd");
        let fd_dir = match fs::read_dir(&fd_path) {
            Ok(d) => d,
            Err(_) => continue, // Permission denied is common
        };

        let info = ProcessInfo {
            pid,
            name: proc_name,
            uid,
        };

        for fd_entry in fd_dir.flatten() {
            // Read the symlink target, looking for "socket:[inode]"
            let link = match fs::read_link(fd_entry.path()) {
                Ok(l) => l,
                Err(_) => continue,
            };

            let link_str = link.to_string_lossy();
            if let Some(inode) = parse_socket_inode(&link_str) {
                map.insert(inode, info.clone());
            }
        }
    }

    map
}

/// Parse "socket:[12345]" into the inode number.
fn parse_socket_inode(s: &str) -> Option<u64> {
    let s = s.strip_prefix("socket:[")?;
    let s = s.strip_suffix(']')?;
    s.parse().ok()
}

/// Read process name from /proc/{pid}/comm.
fn read_comm(pid_path: &Path) -> String {
    fs::read_to_string(pid_path.join("comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string())
}

/// Read UID from /proc/{pid}/status (the Uid: line).
fn read_uid(pid_path: &Path) -> u32 {
    let status = match fs::read_to_string(pid_path.join("status")) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    for line in status.lines() {
        if line.starts_with("Uid:") {
            // Format: Uid:\treal\teffective\tsaved\tfs
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().unwrap_or(0);
            }
        }
    }

    0
}

/// Get the username for a UID. Falls back to the UID string.
pub fn uid_to_name(uid: u32) -> String {
    // Read /etc/passwd for the mapping
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                if let Ok(id) = fields[2].parse::<u32>() {
                    if id == uid {
                        return fields[0].to_string();
                    }
                }
            }
        }
    }
    uid.to_string()
}
