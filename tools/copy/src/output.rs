use std::io::{self, Write};

/// Check if stdout is a TTY.
pub fn is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdout())
}

/// Check if stdin is a TTY (i.e. not piped).
pub fn stdin_is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdin())
}

/// Print a success message (respects quiet/color).
pub fn print_success(msg: &str, quiet: bool, use_color: bool) {
    if quiet {
        return;
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if use_color {
        let _ = writeln!(out, "\x1b[32m✓\x1b[0m {msg}");
    } else {
        let _ = writeln!(out, "✓ {msg}");
    }
}

/// Print an error message to stderr.
pub fn print_error(msg: &str, use_color: bool) {
    let stderr = io::stderr();
    let mut out = stderr.lock();
    if use_color {
        let _ = writeln!(out, "\x1b[31merror:\x1b[0m {msg}");
    } else {
        let _ = writeln!(out, "error: {msg}");
    }
}

/// Print a key-value info line.
#[allow(dead_code)]
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
