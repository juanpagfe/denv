use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;

use chrono::DateTime;

use crate::config::Config;
use crate::format::ArchiveFormat;
use crate::output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Open a tar archive with the appropriate decompressor.
fn open_tar(
    path: &Path,
    format: ArchiveFormat,
) -> Result<tar::Archive<Box<dyn Read>>> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);

    let reader: Box<dyn Read> = match format {
        ArchiveFormat::Tar => Box::new(buf),
        ArchiveFormat::TarGz => Box::new(flate2::read::GzDecoder::new(buf)),
        ArchiveFormat::TarZst => Box::new(zstd::Decoder::new(buf)?),
        ArchiveFormat::TarXz => Box::new(xz2::read::XzDecoder::new(buf)),
        ArchiveFormat::Zip => unreachable!("zip handled separately"),
    };

    Ok(tar::Archive::new(reader))
}

/// Extract a tar-based archive.
pub fn extract(
    archive_path: &Path,
    dest: &Path,
    format: ArchiveFormat,
    specific_files: &[String],
    config: &Config,
) -> Result<()> {
    let mut archive = open_tar(archive_path, format)?;
    let mut extracted = 0u64;

    for entry in archive.entries()? {
        let mut entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: skipping corrupt entry: {e}");
                }
                continue;
            }
        };

        let entry_path = entry.path()?.into_owned();

        // Path traversal protection
        if !super::is_safe_path(&entry_path, dest) {
            eprintln!(
                "warning: skipping unsafe path: {}",
                entry_path.display()
            );
            continue;
        }

        // Filter specific files if requested
        if !specific_files.is_empty() {
            let entry_str = entry_path.to_string_lossy();
            if !specific_files.iter().any(|f| entry_str.starts_with(f)) {
                continue;
            }
        }

        let full_path = dest.join(&entry_path);

        // Handle overwrite behavior
        if full_path.exists() {
            if config.skip_existing {
                if config.verbose {
                    eprintln!("skipping existing: {}", full_path.display());
                }
                continue;
            }
            if !config.overwrite {
                return Err(format!(
                    "file already exists: {} (use --overwrite or --skip-existing)",
                    full_path.display()
                )
                .into());
            }
        }

        entry.set_preserve_permissions(config.preserve_permissions);
        entry.unpack_in(dest)?;
        extracted += 1;

        if config.verbose {
            eprintln!("  {}", entry_path.display());
        }
    }

    if !config.quiet {
        eprintln!("Extracted {} entries to {}", extracted, dest.display());
    }

    Ok(())
}

/// List contents of a tar-based archive.
pub fn list(
    archive_path: &Path,
    format: ArchiveFormat,
    config: &Config,
) -> Result<()> {
    let mut archive = open_tar(archive_path, format)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = !config.no_color && output::is_tty();

    if config.json {
        return list_json(archive_path, format);
    }

    for entry in archive.entries()? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: {e}");
                }
                continue;
            }
        };

        let path = entry.path()?.into_owned();
        let size = entry.size();
        let mtime = entry.header().mtime().unwrap_or(0);
        let mode = entry.header().mode().unwrap_or(0);

        let time_str = DateTime::from_timestamp(mtime as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());

        let perm_str = format_permissions(mode);

        if use_color {
            writeln!(
                out,
                "{} {:>10} {} {}",
                perm_str,
                output::format_bytes(size),
                time_str,
                path.display()
            )?;
        } else {
            writeln!(
                out,
                "{} {:>10} {} {}",
                perm_str,
                output::format_bytes(size),
                time_str,
                path.display()
            )?;
        }
    }

    Ok(())
}

/// Show info/statistics for a tar-based archive.
pub fn info(
    archive_path: &Path,
    archive_size: u64,
    format: ArchiveFormat,
    config: &Config,
) -> Result<()> {
    let mut archive = open_tar(archive_path, format)?;
    let mut file_count = 0u64;
    let mut total_size = 0u64;

    for entry in archive.entries()? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        file_count += 1;
        total_size += entry.size();
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = !config.no_color && output::is_tty();

    if config.json {
        let info = serde_json::json!({
            "format": format.display_name(),
            "files": file_count,
            "original_size": total_size,
            "archive_size": archive_size,
            "compression_ratio": if total_size > 0 {
                format!("{:.1}%", (1.0 - archive_size as f64 / total_size as f64) * 100.0)
            } else {
                "0%".to_string()
            }
        });
        serde_json::to_writer_pretty(&mut out, &info)?;
        writeln!(out)?;
        return Ok(());
    }

    writeln!(out)?;
    output::print_info_line(&mut out, "Format", format.display_name(), use_color)?;
    output::print_info_line(
        &mut out,
        "Files",
        &file_count.to_string(),
        use_color,
    )?;
    output::print_info_line(
        &mut out,
        "Original size",
        &output::format_bytes(total_size),
        use_color,
    )?;
    output::print_info_line(
        &mut out,
        "Archive size",
        &output::format_bytes(archive_size),
        use_color,
    )?;

    if total_size > 0 {
        let ratio = (1.0 - archive_size as f64 / total_size as f64) * 100.0;
        output::print_info_line(
            &mut out,
            "Compression",
            &format!("{ratio:.1}%"),
            use_color,
        )?;
    }

    writeln!(out)?;
    Ok(())
}

/// Verify integrity of a tar-based archive by reading all entries.
pub fn verify(
    archive_path: &Path,
    format: ArchiveFormat,
    config: &Config,
) -> Result<()> {
    let mut archive = open_tar(archive_path, format)?;
    let mut count = 0u64;
    let mut errors = 0u64;

    for entry in archive.entries()? {
        match entry {
            Ok(mut e) => {
                // Read through the entire entry to verify integrity
                let mut sink = io::sink();
                match io::copy(&mut e, &mut sink) {
                    Ok(_) => count += 1,
                    Err(err) => {
                        let path = e.path().map(|p| p.display().to_string())
                            .unwrap_or_else(|_| "<unknown>".to_string());
                        eprintln!("corrupt entry: {path}: {err}");
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("corrupt entry: {e}");
                errors += 1;
            }
        }
    }

    if errors > 0 {
        eprintln!("{count} entries OK, {errors} errors");
        Err(format!("archive has {errors} corrupt entries").into())
    } else {
        if !config.quiet {
            eprintln!("{count} entries OK");
        }
        Ok(())
    }
}

fn list_json(archive_path: &Path, format: ArchiveFormat) -> Result<()> {
    let mut archive = open_tar(archive_path, format)?;
    let mut entries = Vec::new();

    for entry in archive.entries()? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path()?.to_string_lossy().to_string();
        let size = entry.size();
        let mtime = entry.header().mtime().unwrap_or(0);
        let mode = entry.header().mode().unwrap_or(0);

        entries.push(serde_json::json!({
            "path": path,
            "size": size,
            "mtime": mtime,
            "mode": format!("{:o}", mode),
        }));
    }

    let stdout = io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &entries)?;
    println!();
    Ok(())
}

fn format_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    // File type
    s.push(if mode & 0o40000 != 0 { 'd' } else { '-' });
    // Owner
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    // Group
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    // Other
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}
