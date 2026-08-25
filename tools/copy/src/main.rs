mod cli;
mod clipboard;
mod config;
mod filter;
mod history;
mod output;
mod pick;

use std::io::{self, Read, Write};
use std::process;

fn main() {
    let args = cli::parse();
    let config = config::Config::load(&args);
    let use_color = !config.no_color;

    let result = match args.command {
        Some(cli::Command::Copy { trim, lines, file }) => {
            let effective_trim = trim || config.trim;
            cmd_copy(file.as_ref(), effective_trim, lines.as_deref(), &config)
        }
        Some(cli::Command::Paste) => cmd_paste(&config),
        Some(cli::Command::History { count, clear }) => {
            if clear {
                cmd_history_clear(&config)
            } else {
                cmd_history(count, &config)
            }
        }
        Some(cli::Command::Pick) => cmd_pick(&config),
        Some(cli::Command::Clear) => cmd_clear(&config),

        // No subcommand: implicit copy from stdin or file arg
        None => {
            let effective_trim = config.trim;
            cmd_copy(
                args.file.as_ref(),
                effective_trim,
                args.lines.as_deref(),
                &config,
            )
        }
    };

    if let Err(e) = result {
        output::print_error(&e.to_string(), use_color);
        process::exit(1);
    }
}

/// Copy content to clipboard from stdin or a file.
fn cmd_copy(
    file: Option<&std::path::PathBuf>,
    trim: bool,
    lines: Option<&str>,
    config: &config::Config,
) -> io::Result<()> {
    let use_color = !config.no_color;

    // Read input
    let mut content = if let Some(path) = file {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a valid file", path.display()),
            ));
        }
        std::fs::read_to_string(path)?
    } else if !output::stdin_is_tty() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no input provided. Pipe content or pass a file path.\n\
             Usage: copy <file>\n\
             Usage: <command> | copy",
        ));
    };

    // Apply line filter
    if let Some(spec) = lines {
        content = filter::filter_lines(&content, spec)?;
    }

    // Apply trim
    if trim {
        content = content.trim().to_string();
    }

    // Strip trailing newline for clean clipboard content
    if content.ends_with('\n') {
        content.pop();
        if content.ends_with('\r') {
            content.pop();
        }
    }

    let bytes = content.len();
    let line_count = content.lines().count();

    // Save to history (before forking, so parent writes it)
    let mut hist = history::History::load(&config.history_file, config.history_size)?;
    hist.add(content.clone())?;

    // Output (before forking, so parent prints it)
    if config.json {
        let info = serde_json::json!({
            "status": "copied",
            "bytes": bytes,
            "lines": line_count,
        });
        let stdout = io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "{}", serde_json::to_string_pretty(&info).unwrap())?;
    } else if config.verbose {
        output::print_success(
            &format!("copied {bytes} bytes ({line_count} lines) to clipboard"),
            config.quiet,
            use_color,
        );
    } else {
        output::print_success("copied to clipboard", config.quiet, use_color);
    }

    // Set clipboard and fork a daemon to serve it.
    // This must be last — the fork causes the parent to return here
    // while the child stays alive serving clipboard requests.
    clipboard::set_persistent(&content)?;

    Ok(())
}

/// Paste clipboard contents to stdout.
fn cmd_paste(config: &config::Config) -> io::Result<()> {
    let cb = clipboard::Clipboard::new()?;
    let content = cb.get_text()?;

    if config.json {
        let info = serde_json::json!({
            "content": content,
            "bytes": content.len(),
            "lines": content.lines().count(),
        });
        let stdout = io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "{}", serde_json::to_string_pretty(&info).unwrap())?;
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        write!(out, "{content}")?;
        // Add trailing newline if outputting to terminal
        if output::is_tty() {
            writeln!(out)?;
        }
    }

    Ok(())
}

/// Show clipboard history.
fn cmd_history(count: Option<usize>, config: &config::Config) -> io::Result<()> {
    let hist = history::History::load(&config.history_file, config.history_size)?;
    let entries = hist.entries();
    let use_color = !config.no_color;

    if entries.is_empty() {
        if !config.quiet {
            eprintln!("clipboard history is empty");
        }
        return Ok(());
    }

    let limit = count.unwrap_or(20).min(entries.len());

    if config.json {
        let items: Vec<_> = entries[..limit]
            .iter()
            .map(|e| {
                serde_json::json!({
                    "content": e.content,
                    "timestamp": e.timestamp.to_rfc3339(),
                    "bytes": e.bytes,
                    "lines": e.lines,
                })
            })
            .collect();
        let stdout = io::stdout();
        let mut out = stdout.lock();
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&items).unwrap()
        )?;
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for (i, entry) in entries[..limit].iter().enumerate() {
            let preview = preview_content(&entry.content, 60);
            let time = entry.timestamp.format("%Y-%m-%d %H:%M");
            let idx = i + 1;

            if use_color {
                writeln!(
                    out,
                    "  \x1b[33m{idx:>3}\x1b[0m  {preview}  \x1b[90m[{time} · {} bytes]\x1b[0m",
                    entry.bytes
                )?;
            } else {
                writeln!(
                    out,
                    "  {idx:>3}  {preview}  [{time} · {} bytes]",
                    entry.bytes
                )?;
            }
        }
        if entries.len() > limit {
            writeln!(
                out,
                "\n  ... and {} more (use --count or pick to see all)",
                entries.len() - limit
            )?;
        }
    }

    Ok(())
}

/// Clear clipboard history.
fn cmd_history_clear(config: &config::Config) -> io::Result<()> {
    let mut hist = history::History::load(&config.history_file, config.history_size)?;
    hist.clear()?;
    output::print_success("clipboard history cleared", config.quiet, !config.no_color);
    Ok(())
}

/// Clear the clipboard.
fn cmd_clear(config: &config::Config) -> io::Result<()> {
    let cb = clipboard::Clipboard::new()?;
    cb.clear()?;
    output::print_success("clipboard cleared", config.quiet, !config.no_color);
    Ok(())
}

/// Interactive fuzzy picker over clipboard history.
fn cmd_pick(config: &config::Config) -> io::Result<()> {
    let hist = history::History::load(&config.history_file, config.history_size)?;
    let entries = hist.entries();
    let use_color = !config.no_color;

    if entries.is_empty() {
        if !config.quiet {
            eprintln!("clipboard history is empty");
        }
        return Ok(());
    }

    match pick::run(entries, use_color)? {
        Some(idx) => {
            let content = &entries[idx].content;
            output::print_success("selected entry copied to clipboard", config.quiet, use_color);
            clipboard::set_persistent(content)?;
        }
        None => {
            if !config.quiet {
                eprintln!("cancelled");
            }
        }
    }

    Ok(())
}

/// Create a single-line preview of content.
fn preview_content(content: &str, max_len: usize) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let suffix = if content.lines().count() > 1 {
        " …"
    } else {
        ""
    };

    let display = first_line.replace('\t', "  ");
    if display.len() + suffix.len() > max_len {
        let truncated: String = display.chars().take(max_len - 3 - suffix.len()).collect();
        format!("{truncated}...{suffix}")
    } else {
        format!("{display}{suffix}")
    }
}
