use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::config::Config;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Create a zip archive from the given paths.
pub fn create(
    paths: &[PathBuf],
    output_path: &Path,
    config: &Config,
) -> Result<()> {
    let file = File::create(output_path)?;
    let buf = BufWriter::new(file);
    let mut zip = zip::ZipWriter::new(buf);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    // Count files for progress
    let progress = if config.progress {
        let count = count_files(paths);
        let pb = ProgressBar::new(count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files")
                .unwrap()
                .progress_chars("█▓░"),
        );
        Some(pb)
    } else {
        None
    };

    for path in paths {
        if path.is_dir() {
            add_dir_recursive(&mut zip, path, path, options, config, progress.as_ref())?;
        } else if path.is_file() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            add_file(&mut zip, path, &name, options)?;
            if let Some(pb) = &progress {
                pb.inc(1);
            }
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    zip.finish()?;
    Ok(())
}

fn add_dir_recursive<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base: &Path,
    dir: &Path,
    options: SimpleFileOptions,
    config: &Config,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: {e}");
                }
                continue;
            }
        };

        let path = entry.path();
        let relative = path
            .strip_prefix(base.parent().unwrap_or(base))
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            let dir_name = if relative.ends_with('/') {
                relative.clone()
            } else {
                format!("{relative}/")
            };
            zip.add_directory(&dir_name, options)?;
            add_dir_recursive(zip, base, &path, options, config, progress)?;
        } else if path.is_file() {
            add_file(zip, &path, &relative, options)?;
            if let Some(pb) = progress {
                pb.inc(1);
            }
        }
    }

    Ok(())
}

fn add_file<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &Path,
    name: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    zip.start_file(name, options)?;
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        zip.write_all(&buf[..n])?;
    }
    Ok(())
}

fn count_files(paths: &[PathBuf]) -> u64 {
    let mut count = 0u64;
    for path in paths {
        if path.is_file() {
            count += 1;
        } else if path.is_dir() {
            count += count_dir(path);
        }
    }
    count
}

fn count_dir(dir: &Path) -> u64 {
    let mut count = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_dir(&path);
            } else {
                count += 1;
            }
        }
    }
    count
}
