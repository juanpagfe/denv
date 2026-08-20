use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tunnel",
    about = "Create and manage network tunnels (SSH forwarding, SOCKS proxy)",
    version
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// SSH identity file
    #[arg(long, short, global = true)]
    pub identity: Option<PathBuf>,

    /// SSH user
    #[arg(long, global = true)]
    pub user: Option<String>,

    /// SSH port
    #[arg(long, global = true)]
    pub ssh_port: Option<u16>,

    /// Keepalive interval (e.g. "30s")
    #[arg(long, global = true)]
    pub keepalive: Option<String>,

    /// Disable automatic reconnection
    #[arg(long, global = true)]
    pub no_reconnect: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show detailed output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a local port forward (local -> remote via SSH)
    Forward {
        /// Local port to listen on
        local_port: u16,

        /// Remote address (host:port)
        remote: String,

        /// SSH server to tunnel through
        #[arg(long)]
        via: String,
    },

    /// Create a remote port forward (remote -> local via SSH)
    Reverse {
        /// Remote port to listen on
        remote_port: u16,

        /// Local address (host:port)
        local: String,

        /// SSH server to tunnel through
        #[arg(long)]
        via: String,
    },

    /// Create a SOCKS proxy via SSH
    Socks {
        /// Local port for SOCKS proxy
        port: u16,

        /// SSH server to tunnel through
        #[arg(long)]
        via: String,
    },

    /// Start a named tunnel from config
    Start {
        /// Tunnel name (as defined in config)
        name: String,

        /// Run in background
        #[arg(long, short)]
        background: bool,
    },

    /// Stop a named tunnel
    Stop {
        /// Tunnel name
        name: String,
    },

    /// Restart a named tunnel
    Restart {
        /// Tunnel name
        name: String,
    },

    /// Show status of tunnels
    Status {
        /// Specific tunnel name (optional, shows all if omitted)
        name: Option<String>,
    },

    /// List all configured tunnels
    List,

    /// Show logs for a tunnel
    Logs {
        /// Tunnel name
        name: String,
    },

    /// Test connectivity of a tunnel
    Test {
        /// Tunnel name
        name: String,
    },
}

pub fn parse() -> Args {
    Args::parse()
}

/// Parse a keepalive/interval string like "30s", "1m" into seconds.
pub fn parse_duration_secs(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    if s.ends_with('m') {
        s[..s.len() - 1]
            .trim()
            .parse::<u64>()
            .map(|v| v * 60)
            .map_err(|_| format!("invalid duration: {s}"))
    } else if s.ends_with('s') {
        s[..s.len() - 1]
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid duration: {s}"))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("invalid duration: {s}"))
    }
}
