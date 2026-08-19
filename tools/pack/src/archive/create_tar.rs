use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};

use crate::config::Config;
use crate::format::ArchiveFormat;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Create a tar-based archive (plain, gzip, zstd, or xz compressed).
pub fn create(
    paths: &[PathBuf],
    output_path: &Path,
    format: ArchiveFormat,
    config: &Config,
) -> Result<()> {
    let file = File::create(output_path)?;
    let buf = BufWriter::new(file);

    match format {
        ArchiveFormat::Tar => {
            let mut builder = tar::Builder::new(buf);
            add_paths(&mut builder, paths, config)?;
            builder.finish()?;
        }
        ArchiveFormat::TarGz => {
            let level = config
                .compression_level
                .unwrap_or(format.default_compression());
            let encoder = flate2::write::GzEncoder::new(
                buf,
                flate2::Compression::new(level as u32),
            );
            let mut builder = tar::Builder::new(encoder);
            add_paths(&mut builder, paths, config)?;
            let encoder = builder.into_inner()?;
            encoder.finish()?;
        }
        ArchiveFormat::TarZst => {
            let level = config
                .compression_level
                .unwrap_or(format.default_compression());
            let encoder = zstd::Encoder::new(buf, level)?;
            let mut builder = tar::Builder::new(encoder);
            add_paths(&mut builder, paths, config)?;
            let encoder = builder.into_inner()?;
            encoder.finish()?;
        }
        ArchiveFormat::TarXz => {
            let level = config
                .compression_level
                .unwrap_or(format.default_compression()) as u32;
            let encoder = xz2::write::XzEncoder::new(buf, level);
            let mut builder = tar::Builder::new(encoder);
            add_paths(&mut builder, paths, config)?;
            let encoder = builder.into_inner()?;
            encoder.finish()?;
        }
        ArchiveFormat::Zip => unreachable!("zip handled separately"),
    }

    Ok(())
}

/// Add paths to a tar builder, recursing into directories.
fn add_paths<W: Write>(
    builder: &mut tar::Builder<W>,
    paths: &[PathBuf],
    config: &Config,
) -> Result<()> {
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
            add_dir_recursive(builder, path, path, config, progress.as_ref())?;
        } else if path.is_file() {
            let name = path
                .file_name()
                .map(|n| PathBuf::from(n))
                .unwrap_or_else(|| path.clone());
            builder.append_path_with_name(path, &name)?;
            if let Some(pb) = &progress {
                pb.inc(1);
            }
        } else if path.is_symlink() {
            let name = path
                .file_name()
                .map(|n| PathBuf::from(n))
                .unwrap_or_else(|| path.clone());
            builder.append_path_with_name(path, &name)?;
            if let Some(pb) = &progress {
                pb.inc(1);
            }
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    Ok(())
}

/// Recursively add a directory to the tar archive.
fn add_dir_recursive<W: Write>(
    builder: &mut tar::Builder<W>,
    base: &Path,
    dir: &Path,
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
        let relative = path.strip_prefix(base.parent().unwrap_or(base))
            .unwrap_or(&path);

        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: cannot stat {}: {e}", path.display());
                }
                continue;
            }
        };

        if metadata.is_dir() {
            // Add directory entry itself
            let mut header = tar::Header::new_gnu();
            header.set_metadata_in_mode(&metadata, tar::HeaderMode::Deterministic);
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            builder.append_data(&mut header, relative, io::empty())?;

            add_dir_recursive(builder, base, &path, config, progress)?;
        } else if metadata.is_symlink() {
            let target = fs::read_link(&path)?;
            let mut header = tar::Header::new_gnu();
            header.set_metadata_in_mode(&metadata, tar::HeaderMode::Deterministic);
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            builder.append_link(&mut header, relative, &target)?;
            if let Some(pb) = progress {
                pb.inc(1);
            }
        } else if metadata.is_file() {
            builder.append_path_with_name(&path, relative)?;
            if let Some(pb) = progress {
                pb.inc(1);
            }
        }
    }

    Ok(())
}

/// Count total files for progress bar.
fn count_files(paths: &[PathBuf]) -> u64 {
    let mut count = 0u64;
    for path in paths {
        if path.is_file() || path.is_symlink() {
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
