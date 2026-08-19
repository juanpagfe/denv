use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::config::Config;
use crate::format::ArchiveFormat;
use crate::output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Extract a zip archive.
pub fn extract(
    archive_path: &Path,
    dest: &Path,
    specific_files: &[String],
    config: &Config,
) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut extracted = 0u64;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => {
                eprintln!("warning: skipping unsafe path in archive");
                continue;
            }
        };

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
        if full_path.exists() && !entry.is_dir() {
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

        if entry.is_dir() {
            fs::create_dir_all(&full_path)?;
        } else {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&full_path)?;
            io::copy(&mut entry, &mut outfile)?;

            // Set permissions on unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    fs::set_permissions(&full_path, fs::Permissions::from_mode(mode))?;
                }
            }

            extracted += 1;
        }

        if config.verbose {
            eprintln!("  {}", entry_path.display());
        }
    }

    if !config.quiet {
        eprintln!("Extracted {} files to {}", extracted, dest.display());
    }

    Ok(())
}

/// List contents of a zip archive.
pub fn list(archive_path: &Path, config: &Config) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if config.json {
        return list_json(archive_path);
    }

    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i)?;
        let path = entry.name();
        let size = entry.size();
        let compressed = entry.compressed_size();

        let time_str = entry
            .last_modified()
            .and_then(|dt| {
                let date = NaiveDate::from_ymd_opt(
                    dt.year() as i32,
                    dt.month() as u32,
                    dt.day() as u32,
                )?;
                let time = NaiveTime::from_hms_opt(
                    dt.hour() as u32,
                    dt.minute() as u32,
                    dt.second() as u32,
                )?;
                Some(NaiveDateTime::new(date, time).format("%Y-%m-%d %H:%M").to_string())
            })
            .unwrap_or_else(|| "-".to_string());

        let perm_str = entry
            .unix_mode()
            .map(format_permissions)
            .unwrap_or_else(|| "----------".to_string());

        writeln!(
            out,
            "{} {:>10} {:>10} {} {}",
            perm_str,
            output::format_bytes(size),
            output::format_bytes(compressed),
            time_str,
            path,
        )?;
    }

    Ok(())
}

/// Show info/statistics for a zip archive.
pub fn info(
    archive_path: &Path,
    archive_size: u64,
    format: ArchiveFormat,
    config: &Config,
) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut file_count = 0u64;
    let mut total_size = 0u64;

    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i)?;
        if !entry.is_dir() {
            file_count += 1;
            total_size += entry.size();
        }
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
    output::print_info_line(&mut out, "Files", &file_count.to_string(), use_color)?;
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
        output::print_info_line(&mut out, "Compression", &format!("{ratio:.1}%"), use_color)?;
    }

    writeln!(out)?;
    Ok(())
}

/// Verify integrity of a zip archive.
pub fn verify(archive_path: &Path, config: &Config) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut count = 0u64;
    let mut errors = 0u64;

    for i in 0..archive.len() {
        match archive.by_index(i) {
            Ok(mut entry) => {
                let name = entry.name().to_string();
                let mut sink = io::sink();
                match io::copy(&mut entry, &mut sink) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        eprintln!("corrupt entry: {name}: {e}");
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("corrupt entry #{i}: {e}");
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

fn list_json(archive_path: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i)?;
        entries.push(serde_json::json!({
            "path": entry.name(),
            "size": entry.size(),
            "compressed_size": entry.compressed_size(),
            "mode": entry.unix_mode().map(|m| format!("{:o}", m)),
        }));
    }

    let stdout = io::stdout();
    serde_json::to_writer_pretty(stdout.lock(), &entries)?;
    println!();
    Ok(())
}

fn format_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    s.push(if mode & 0o40000 != 0 { 'd' } else { '-' });
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}
