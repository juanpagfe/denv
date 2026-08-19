use std::io::{self, Write};
use std::time::Duration;

use crate::config::Config;
use crate::scanner::DuplicateGroup;

/// Display scan results based on configured output format.
pub fn display_results(
    groups: &[DuplicateGroup],
    stats: &ScanStats,
    config: &Config,
) -> io::Result<()> {
    if config.json {
        print_json(groups, stats)?;
    } else if config.csv {
        print_csv(groups)?;
    } else {
        print_table(groups, stats, config)?;
    }
    Ok(())
}

pub struct ScanStats {
    pub files_scanned: u64,
    pub duplicate_groups: u64,
    pub duplicate_files: u64,
    pub wasted_space: u64,
    pub elapsed: Duration,
}

fn print_table(
    groups: &[DuplicateGroup],
    stats: &ScanStats,
    config: &Config,
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if groups.is_empty() {
        if !config.quiet {
            writeln!(out, "No duplicates found.")?;
        }
        return Ok(());
    }

    let use_color = !config.no_color && atty_stdout();

    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }

        let group_size: u64 = group.files.iter().skip(1).map(|f| f.size).sum();
        let header = format!(
            "Duplicate group #{} — {} wasted",
            i + 1,
            format_bytes(group_size)
        );

        if use_color {
            writeln!(out, "\x1b[1;33m{header}\x1b[0m")?;
        } else {
            writeln!(out, "{header}")?;
        }
        writeln!(out)?;

        for file in &group.files {
            writeln!(
                out,
                "  {:>10}  {}",
                format_bytes(file.size),
                file.path.display()
            )?;
        }
    }

    if !config.quiet {
        writeln!(out)?;
        print_stats(&mut out, stats, use_color)?;
    }

    Ok(())
}

fn print_stats(out: &mut impl Write, stats: &ScanStats, use_color: bool) -> io::Result<()> {
    let sep = if use_color {
        "\x1b[2m──────────────────────────────────\x1b[0m"
    } else {
        "──────────────────────────────────"
    };
    writeln!(out, "{sep}")?;
    writeln!(out, "Files scanned:    {}", stats.files_scanned)?;
    writeln!(out, "Duplicate groups: {}", stats.duplicate_groups)?;
    writeln!(out, "Duplicate files:  {}", stats.duplicate_files)?;
    writeln!(
        out,
        "Wasted space:     {}",
        format_bytes(stats.wasted_space)
    )?;
    writeln!(out, "Elapsed:          {:.2?}", stats.elapsed)?;
    Ok(())
}

fn print_json(groups: &[DuplicateGroup], stats: &ScanStats) -> io::Result<()> {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        groups: Vec<JsonGroup>,
        stats: JsonStats,
    }

    #[derive(serde::Serialize)]
    struct JsonGroup {
        hash: String,
        size: u64,
        wasted: u64,
        files: Vec<String>,
    }

    #[derive(serde::Serialize)]
    struct JsonStats {
        files_scanned: u64,
        duplicate_groups: u64,
        duplicate_files: u64,
        wasted_bytes: u64,
        elapsed_seconds: f64,
    }

    let output = JsonOutput {
        groups: groups
            .iter()
            .map(|g| {
                let wasted: u64 = g.files.iter().skip(1).map(|f| f.size).sum();
                JsonGroup {
                    hash: g.hash.clone(),
                    size: g.files.first().map(|f| f.size).unwrap_or(0),
                    wasted,
                    files: g.files.iter().map(|f| f.path.display().to_string()).collect(),
                }
            })
            .collect(),
        stats: JsonStats {
            files_scanned: stats.files_scanned,
            duplicate_groups: stats.duplicate_groups,
            duplicate_files: stats.duplicate_files,
            wasted_bytes: stats.wasted_space,
            elapsed_seconds: stats.elapsed.as_secs_f64(),
        },
    };

    let stdout = io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &output)?;
    println!();
    Ok(())
}

fn print_csv(groups: &[DuplicateGroup]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut wtr = csv::Writer::from_writer(stdout.lock());
    wtr.write_record(["group", "hash", "size", "path"])?;

    for (i, group) in groups.iter().enumerate() {
        for file in &group.files {
            wtr.write_record(&[
                (i + 1).to_string(),
                group.hash.clone(),
                file.size.to_string(),
                file.path.display().to_string(),
            ])?;
        }
    }
    wtr.flush()?;
    Ok(())
}

fn atty_stdout() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdout())
}

/// Format byte count into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    bytesize::ByteSize(bytes).to_string()
}
