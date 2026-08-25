use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "copy",
    about = "Self-contained clipboard manager: copy, paste, history, and interactive picker",
    version,
    arg_required_else_help = false
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

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show detailed output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    // ── Flags that apply to the implicit copy command ──

    /// Strip leading/trailing whitespace before copying
    #[arg(long, global = true)]
    pub trim: bool,

    /// Copy only specific lines (e.g. "5-10", "3", "1-")
    #[arg(long, global = true)]
    pub lines: Option<String>,

    /// File to copy content from (when used without a subcommand)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Copy content to clipboard from stdin or a file
    Copy {
        /// Strip leading/trailing whitespace before copying
        #[arg(long)]
        trim: bool,

        /// Copy only specific lines (e.g. "5-10", "3", "1-")
        #[arg(long)]
        lines: Option<String>,

        /// File to copy content from (reads stdin if omitted)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Paste clipboard contents to stdout
    Paste,

    /// Show clipboard history
    History {
        /// Number of entries to show (default: from config or 20)
        #[arg(short, long)]
        count: Option<usize>,

        /// Clear all history
        #[arg(long)]
        clear: bool,
    },

    /// Interactive fuzzy picker over clipboard history
    Pick,

    /// Clear the current clipboard contents
    Clear,
}

pub fn parse() -> Args {
    Args::parse()
}
