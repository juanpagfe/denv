use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "pack",
    about = "Create, extract, list, verify and inspect archives",
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

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show detailed output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Show progress bar
    #[arg(long, global = true)]
    pub progress: bool,

    /// Compression level (1-22 for zstd, 1-9 for gzip/xz)
    #[arg(long, global = true)]
    pub compression: Option<i32>,

    /// Force a specific format instead of auto-detecting
    #[arg(long, global = true, value_enum)]
    pub format: Option<FormatChoice>,

    /// Overwrite existing files during extraction
    #[arg(long, global = true)]
    pub overwrite: bool,

    /// Skip existing files during extraction
    #[arg(long, global = true)]
    pub skip_existing: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new archive from files and directories
    Create {
        /// Output archive file path
        #[arg(required = true)]
        output_file: PathBuf,

        /// Files and directories to add
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },

    /// Extract an archive
    Extract {
        /// Archive file to extract
        #[arg(required = true)]
        archive: PathBuf,

        /// Output directory (defaults to current directory)
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Extract only specific files (optional)
        #[arg()]
        files: Vec<String>,
    },

    /// List contents of an archive
    List {
        /// Archive file to list
        #[arg(required = true)]
        archive: PathBuf,
    },

    /// Show archive metadata and statistics
    Info {
        /// Archive file to inspect
        #[arg(required = true)]
        archive: PathBuf,
    },

    /// Verify archive integrity
    Verify {
        /// Archive file to verify
        #[arg(required = true)]
        archive: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
pub enum FormatChoice {
    Tar,
    TarGz,
    TarZst,
    TarXz,
    Zip,
}

pub fn parse() -> Args {
    Args::parse()
}
