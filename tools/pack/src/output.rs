use std::io::{self, Write};

/// Format byte count into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    bytesize::ByteSize(bytes).to_string()
}

/// Check if stdout is a TTY.
pub fn is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdout())
}

/// Print a key-value info line.
pub fn print_info_line(
    out: &mut impl Write,
    key: &str,
    value: &str,
    use_color: bool,
) -> io::Result<()> {
    if use_color {
        writeln!(out, "  \x1b[1m{:<18}\x1b[0m {}", key, value)
    } else {
        writeln!(out, "  {:<18} {}", key, value)
    }
}
