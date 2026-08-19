use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::cli::{self, Args, HashAlgorithm};

/// Runtime configuration merged from config file + CLI args.
pub struct Config {
    pub hash_algorithm: HashChoice,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub workers: usize,
    pub json: bool,
    pub csv: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub no_color: bool,
}

#[derive(Clone, Copy)]
pub enum HashChoice {
    Blake3,
    Sha256,
}

/// Deserialized from ~/.config/dupe/config.yaml
#[derive(Deserialize, Default)]
struct FileConfig {
    hash: Option<String>,
    min_size: Option<String>,
    follow_symlinks: Option<bool>,

    #[serde(default)]
    exclude: Vec<String>,

    scan: Option<ScanConfig>,
}

#[derive(Deserialize, Default)]
struct ScanConfig {
    workers: Option<usize>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let file_cfg = load_file_config(args.config.as_deref());

        // CLI flags override file config, file config overrides defaults
        let hash_algorithm = match &args.hash {
            Some(HashAlgorithm::Sha256) => HashChoice::Sha256,
            Some(HashAlgorithm::Blake3) => HashChoice::Blake3,
            None => match file_cfg.hash.as_deref() {
                Some("sha256") | Some("SHA256") | Some("sha-256") => HashChoice::Sha256,
                _ => HashChoice::Blake3,
            },
        };

        let min_size = args
            .min_size
            .as_deref()
            .map(cli::parse_size)
            .transpose()
            .unwrap_or(None)
            .or_else(|| {
                file_cfg
                    .min_size
                    .as_deref()
                    .and_then(|s| cli::parse_size(s).ok())
            });

        let max_size = args
            .max_size
            .as_deref()
            .map(cli::parse_size)
            .transpose()
            .unwrap_or(None);

        let follow_symlinks = args.follow_symlinks || file_cfg.follow_symlinks.unwrap_or(false);

        // Merge exclude dirs from CLI and config file
        let mut exclude_dirs = args.exclude_dir.clone();
        for dir in &file_cfg.exclude {
            if !exclude_dirs.contains(dir) {
                exclude_dirs.push(dir.clone());
            }
        }

        let workers = args
            .workers
            .or(file_cfg.scan.as_ref().and_then(|s| s.workers))
            .unwrap_or_else(num_cpus::get);

        let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();

        Config {
            hash_algorithm,
            min_size,
            max_size,
            include_patterns: args.include.clone(),
            exclude_patterns: args.exclude.clone(),
            exclude_dirs,
            max_depth: args.depth,
            follow_symlinks,
            workers,
            json: args.json,
            csv: args.csv,
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
        .join("dupe")
        .join("config.yaml")
}
