use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "netwatch",
    about = "Monitor network activity by process, host, and port",
    version
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Output as CSV
    #[arg(long, global = true)]
    pub csv: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show detailed output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Refresh interval (e.g. "1s", "500ms")
    #[arg(long, global = true)]
    pub interval: Option<String>,

    /// Resolve remote IPs to hostnames
    #[arg(long, global = true)]
    pub resolve: bool,

    /// Disable DNS resolution
    #[arg(long, global = true)]
    pub no_resolve: bool,

    /// Filter by process name or PID
    #[arg(long, global = true)]
    pub process: Option<String>,

    /// Filter by user
    #[arg(long, global = true)]
    pub user: Option<String>,

    /// Filter by remote host
    #[arg(long, global = true)]
    pub host: Option<String>,

    /// Filter by port
    #[arg(long, global = true)]
    pub port: Option<u16>,

    /// Filter by protocol (tcp, udp)
    #[arg(long, global = true)]
    pub protocol: Option<String>,

    /// Show only established connections
    #[arg(long, global = true)]
    pub established: bool,

    /// Show only listening sockets
    #[arg(long, global = true)]
    pub listening: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show top connections sorted by activity (default)
    Top,

    /// List all connections
    Connections,

    /// Show connections for a specific process
    Process {
        /// Process PID
        pid: u32,
    },

    /// Show connections to/from a host
    Host {
        /// Hostname or IP
        host: String,
    },

    /// Show connections on a port
    Port {
        /// Port number
        port: u16,
    },

    /// Continuous watch mode (TUI)
    Watch,
}

pub fn parse() -> Args {
    Args::parse()
}

/// Parse an interval string like "1s", "500ms", "2s" into milliseconds.
pub fn parse_interval_ms(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    if s.ends_with("ms") {
        s[..s.len() - 2]
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid interval: {s}"))
    } else if s.ends_with('s') {
        s[..s.len() - 1]
            .trim()
            .parse::<f64>()
            .map(|v| (v * 1000.0) as u64)
            .map_err(|_| format!("invalid interval: {s}"))
    } else {
        s.parse::<u64>()
            .map(|v| v * 1000) // Assume seconds if no suffix
            .map_err(|_| format!("invalid interval: {s}"))
    }
}
