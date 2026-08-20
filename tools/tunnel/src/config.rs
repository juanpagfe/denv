use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cli::{self, Args};

/// Runtime configuration.
pub struct Config {
    pub identity_file: Option<PathBuf>,
    pub ssh_user: Option<String>,
    pub ssh_port: u16,
    pub keepalive_secs: u64,
    pub reconnect: bool,
    pub reconnect_delay_secs: u64,
    pub reconnect_max_delay_secs: u64,
    pub quiet: bool,
    #[allow(dead_code)]
    pub verbose: bool,
    #[allow(dead_code)]
    pub no_color: bool,
    pub json: bool,
    pub tunnels: HashMap<String, TunnelDef>,
    pub pid_dir: PathBuf,
    pub log_dir: PathBuf,
}

/// A named tunnel definition from the config file.
#[derive(Debug, Clone, Deserialize)]
pub struct TunnelDef {
    #[serde(rename = "type")]
    pub tunnel_type: String,
    pub local: Option<String>,
    pub remote: Option<String>,
    pub port: Option<u16>,

    #[serde(default)]
    pub ssh: SshConfig,

    #[serde(default)]
    #[allow(dead_code)]
    pub reconnect: ReconnectConfig,

    #[allow(dead_code)]
    pub keepalive: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SshConfig {
    pub host: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ReconnectConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_delay")]
    pub delay: String,
    #[serde(default = "default_max_delay")]
    pub max_delay: String,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            delay: "5s".to_string(),
            max_delay: "60s".to_string(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_delay() -> String {
    "5s".to_string()
}
fn default_max_delay() -> String {
    "60s".to_string()
}

/// Deserialized from ~/.config/tunnel/config.yaml
#[derive(Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    tunnels: HashMap<String, TunnelDef>,

    ssh: Option<GlobalSshConfig>,
}

#[derive(Deserialize, Default)]
struct GlobalSshConfig {
    user: Option<String>,
    port: Option<u16>,
    identity_file: Option<String>,
    keepalive: Option<String>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let file_cfg = load_file_config(args.config.as_deref());

        let ssh_port = args
            .ssh_port
            .or(file_cfg.ssh.as_ref().and_then(|s| s.port))
            .unwrap_or(22);

        let identity_file = args
            .identity
            .clone()
            .or_else(|| {
                file_cfg
                    .ssh
                    .as_ref()
                    .and_then(|s| s.identity_file.as_ref())
                    .map(|s| PathBuf::from(shellexpand(s)))
            });

        let ssh_user = args.user.clone().or_else(|| {
            file_cfg.ssh.as_ref().and_then(|s| s.user.clone())
        });

        let keepalive_secs = args
            .keepalive
            .as_deref()
            .map(cli::parse_duration_secs)
            .transpose()
            .unwrap_or(None)
            .or_else(|| {
                file_cfg
                    .ssh
                    .as_ref()
                    .and_then(|s| s.keepalive.as_deref())
                    .and_then(|s| cli::parse_duration_secs(s).ok())
            })
            .unwrap_or(30);

        let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();

        let runtime_dir = dirs::runtime_dir()
            .or_else(|| dirs::state_dir())
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("tunnel");

        let log_dir = dirs::state_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".local/state")
            })
            .join("tunnel/logs");

        Config {
            identity_file,
            ssh_user,
            ssh_port,
            keepalive_secs,
            reconnect: !args.no_reconnect,
            reconnect_delay_secs: 5,
            reconnect_max_delay_secs: 60,
            quiet: args.quiet,
            verbose: args.verbose,
            no_color,
            json: args.json,
            tunnels: file_cfg.tunnels,
            pid_dir: runtime_dir,
            log_dir,
        }
    }

    /// Get a named tunnel definition.
    pub fn get_tunnel(&self, name: &str) -> Option<&TunnelDef> {
        self.tunnels.get(name)
    }
}

fn load_file_config(explicit_path: Option<&Path>) -> FileConfig {
    let path = if let Some(p) = explicit_path {
        p.to_path_buf()
    } else {
        default_config_path()
    };

    if !path.exists() {
        return FileConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("warning: failed to parse {}: {e}", path.display());
            FileConfig::default()
        }),
        Err(e) => {
            eprintln!("warning: failed to read {}: {e}", path.display());
            FileConfig::default()
        }
    }
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("tunnel")
        .join("config.yaml")
}

/// Expand ~ in paths (public for use in manager).
pub fn shellexpand_pub(s: &str) -> String {
    shellexpand(s)
}

fn shellexpand(s: &str) -> String {
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}
