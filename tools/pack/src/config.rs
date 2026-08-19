use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::cli::{Args, FormatChoice};

/// Runtime configuration merged from config file + CLI args.
pub struct Config {
    pub compression_level: Option<i32>,
    pub default_format: String,
    pub format_override: Option<FormatChoice>,
    pub progress: bool,
    pub overwrite: bool,
    pub skip_existing: bool,
    pub preserve_permissions: bool,
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub no_color: bool,
}

/// Deserialized from ~/.config/pack/config.yaml
#[derive(Deserialize, Default)]
struct FileConfig {
    format: Option<String>,

    compression: Option<CompressionConfig>,
    output: Option<OutputConfig>,
    extract: Option<ExtractConfig>,
}

#[derive(Deserialize, Default)]
struct CompressionConfig {
    level: Option<i32>,
}

#[derive(Deserialize, Default)]
struct OutputConfig {
    progress: Option<bool>,
}

#[derive(Deserialize, Default)]
struct ExtractConfig {
    overwrite: Option<bool>,
    preserve_permissions: Option<bool>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let file_cfg = load_file_config(args.config.as_deref());

        let compression_level = args.compression.or_else(|| {
            file_cfg.compression.as_ref().and_then(|c| c.level)
        });

        let progress = args.progress
            || file_cfg.output.as_ref().and_then(|o| o.progress).unwrap_or(false);

        let overwrite = args.overwrite
            || file_cfg.extract.as_ref().and_then(|e| e.overwrite).unwrap_or(false);

        let preserve_permissions = file_cfg
            .extract
            .as_ref()
            .and_then(|e| e.preserve_permissions)
            .unwrap_or(true);

        let default_format = file_cfg.format.unwrap_or_else(|| "tar.zst".to_string());

        let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();

        Config {
            compression_level,
            default_format,
            format_override: args.format.clone(),
            progress,
            overwrite,
            skip_existing: args.skip_existing,
            preserve_permissions,
            json: args.json,
            quiet: args.quiet,
            verbose: args.verbose,
            no_color,
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
        .join("pack")
        .join("config.yaml")
}
