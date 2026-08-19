use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dupe",
    about = "Find and manage duplicate files efficiently and safely",
    version
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

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

    /// Minimum file size to consider
    #[arg(long, global = true)]
    pub min_size: Option<String>,

    /// Maximum file size to consider
    #[arg(long, global = true)]
    pub max_size: Option<String>,

    /// Include only files matching pattern (e.g. '*.jpg')
    #[arg(long, global = true, action = clap::ArgAction::Append)]
    pub include: Vec<String>,

    /// Exclude files matching pattern (e.g. '*.tmp')
    #[arg(long, global = true, action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Exclude directories by name
    #[arg(long, global = true, action = clap::ArgAction::Append)]
    pub exclude_dir: Vec<String>,

    /// Maximum recursion depth
    #[arg(long, global = true)]
    pub depth: Option<usize>,

    /// Follow symlinks (disabled by default)
    #[arg(long, global = true)]
    pub follow_symlinks: bool,

    /// Hash algorithm to use
    #[arg(long, global = true, value_enum)]
    pub hash: Option<HashAlgorithm>,

    /// Number of worker threads
    #[arg(long, global = true)]
    pub workers: Option<usize>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scan directories for duplicate files (default)
    Scan {
        /// Directories to scan
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },

    /// Interactively select which duplicates to remove
    Clean {
        /// Directories to scan
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
}

/// Parse CLI arguments. When invoked without a subcommand (just paths),
/// default to the Scan command.
pub fn parse() -> Args {
    // Try normal parse first
    let result = Args::try_parse();
    match result {
        Ok(args) => args,
        Err(_) => {
            // If the user ran `dupe ~/Downloads` or `dupe --flag ~/Downloads`
            // without a subcommand, re-parse by injecting "scan" just before
            // the first positional argument (non-flag, non-flag-value).
            let raw_args: Vec<String> = std::env::args().collect();
            let known = ["scan", "clean", "help"];

            // Check if any arg is a known subcommand
            let has_subcommand = raw_args.iter().skip(1).any(|a| known.contains(&a.as_str()));

            if !has_subcommand && raw_args.len() > 1 {
                // Find where to insert "scan": after all flags and their values
                let mut insert_pos = None;
                let mut skip_next = false;
                let flags_with_value = [
                    "--config", "--min-size", "--max-size", "--include",
                    "--exclude", "--exclude-dir", "--depth", "--hash", "--workers",
                ];

                for (i, arg) in raw_args.iter().enumerate().skip(1) {
                    if skip_next {
                        skip_next = false;
                        continue;
                    }
                    if arg.starts_with('-') {
                        // Check if this flag expects a value as next arg
                        if flags_with_value.contains(&arg.as_str()) && !arg.contains('=') {
                            skip_next = true;
                        }
                        continue;
                    }
                    // First positional arg found
                    insert_pos = Some(i);
                    break;
                }

                if let Some(pos) = insert_pos {
                    let mut patched = raw_args[..pos].to_vec();
                    patched.push("scan".to_string());
                    patched.extend_from_slice(&raw_args[pos..]);
                    return Args::parse_from(patched);
                }
            }

            // Re-run to get the proper error message
            Args::parse();
            unreachable!()
        }
    }
}


/// Parse a human-readable size string like "10MB", "1GB", "500KB" into bytes.
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with("TB") {
        (&s[..s.len() - 2], 1_000_000_000_000u64)
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], 1_000_000_000u64)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1_000_000u64)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1_000u64)
    } else if s.ends_with("B") {
        (&s[..s.len() - 1], 1u64)
    } else {
        // Assume bytes if no suffix
        (s.as_str(), 1u64)
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("invalid size: {s}"))?;
    Ok((num * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("10MB").unwrap(), 10_000_000);
        assert_eq!(parse_size("1GB").unwrap(), 1_000_000_000);
        assert_eq!(parse_size("500KB").unwrap(), 500_000);
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("1.5GB").unwrap(), 1_500_000_000);
    }
}
