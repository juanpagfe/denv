use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::cli::{self, Args};

/// Runtime configuration.
pub struct Config {
    pub interval_ms: u64,
    pub resolve_dns: bool,
    pub json: bool,
    pub csv: bool,
    pub quiet: bool,
    #[allow(dead_code)] // Used in future enhancements
    pub verbose: bool,
    pub no_color: bool,
    pub filter_pid: Option<u32>,
    pub filter_process: Option<String>,
    pub filter_user: Option<String>,
    pub filter_host: Option<String>,
    pub filter_port: Option<u16>,
    pub filter_protocol: Option<String>,
    pub only_established: bool,
    pub only_listening: bool,
    pub max_connections: usize,
}

#[derive(Deserialize, Default)]
struct FileConfig {
    interval: Option<String>,
    resolve_hosts: Option<bool>,

    filters: Option<FiltersConfig>,
    ui: Option<UiConfig>,
}

#[derive(Deserialize, Default)]
struct FiltersConfig {
    protocols: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
struct UiConfig {
    color: Option<bool>,
    max_connections: Option<usize>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let file_cfg = load_file_config(args.config.as_deref());

        let interval_ms = args
            .interval
            .as_deref()
            .map(cli::parse_interval_ms)
            .transpose()
            .unwrap_or(None)
            .or_else(|| {
                file_cfg
                    .interval
                    .as_deref()
                    .and_then(|s| cli::parse_interval_ms(s).ok())
            })
            .unwrap_or(1000);

        let resolve_dns = if args.no_resolve {
            false
        } else if args.resolve {
            true
        } else {
            file_cfg.resolve_hosts.unwrap_or(false)
        };

        let no_color = args.no_color
            || std::env::var("NO_COLOR").is_ok()
            || file_cfg.ui.as_ref().and_then(|u| u.color).map(|c| !c).unwrap_or(false);

        let max_connections = file_cfg
            .ui
            .as_ref()
            .and_then(|u| u.max_connections)
            .unwrap_or(200);

        // Parse --process as either PID or process name
        let (filter_pid, filter_process) = if let Some(ref p) = args.process {
            if let Ok(pid) = p.parse::<u32>() {
                (Some(pid), None)
            } else {
                (None, Some(p.clone()))
            }
        } else {
            (None, None)
        };

        let filter_protocol = args.protocol.clone().or_else(|| {
            file_cfg
                .filters
                .as_ref()
                .and_then(|f| f.protocols.as_ref())
                .and_then(|p| p.first().cloned())
        });

        Config {
            interval_ms,
            resolve_dns,
            json: args.json,
            csv: args.csv,
            quiet: args.quiet,
            verbose: args.verbose,
            no_color,
            filter_pid: filter_pid.or(args.port.and(None)),
            filter_process,
            filter_user: args.user.clone(),
            filter_host: args.host.clone(),
            filter_port: args.port,
            filter_protocol,
            only_established: args.established,
            only_listening: args.listening,
            max_connections,
        }
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
        .join("netwatch")
        .join("config.yaml")
}
