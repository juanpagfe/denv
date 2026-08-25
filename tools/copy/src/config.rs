use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::cli::Args;

/// Runtime configuration merged from config file + CLI args.
pub struct Config {
    pub history_size: usize,
    pub history_file: PathBuf,
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub no_color: bool,
    pub trim: bool,
}

/// Deserialized from ~/.config/copy/config.yaml
#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct FileConfig {
    history: Option<HistoryConfig>,
    output: Option<OutputConfig>,
    clipboard: Option<ClipboardConfig>,
}

#[derive(Deserialize, Default)]
struct HistoryConfig {
    /// Maximum number of entries to keep
    size: Option<usize>,
    /// Custom path to history file
    file: Option<String>,
}

#[derive(Deserialize, Default)]
struct OutputConfig {
    /// Always strip whitespace
    trim: Option<bool>,
}

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct ClipboardConfig {
    /// Set both primary and clipboard selections (default: true)
    set_primary: Option<bool>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let file_cfg = load_file_config(args.config.as_deref());

        let default_history_path = data_dir().join("history.json");

        let history_file = file_cfg
            .history
            .as_ref()
            .and_then(|h| h.file.as_ref())
            .map(PathBuf::from)
            .unwrap_or(default_history_path);

        let history_size = file_cfg
            .history
            .as_ref()
            .and_then(|h| h.size)
            .unwrap_or(500);

        let config_trim = file_cfg
            .output
            .as_ref()
            .and_then(|o| o.trim)
            .unwrap_or(false);

        let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();

        Config {
            history_size,
            history_file,
            json: args.json,
            quiet: args.quiet,
            verbose: args.verbose,
            no_color,
            trim: args.trim || config_trim,
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
        .join("copy")
        .join("config.yaml")
}

/// Data directory for history storage.
pub fn data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("copy");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}
