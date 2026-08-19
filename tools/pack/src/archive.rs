mod create_tar;
mod create_zip;
mod extract_tar;
mod extract_zip;

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::format::ArchiveFormat;
use crate::output;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Create an archive from the given paths.
pub fn create(
    paths: &[PathBuf],
    output_file: &Path,
    config: &Config,
) -> Result<()> {
    let format = resolve_format(Some(output_file), config)?;

    // Verify all source paths exist
    for path in paths {
        if !path.exists() {
            return Err(format!("path does not exist: {}", path.display()).into());
        }
    }

    // Use atomic creation: write to tmp file, then rename
    let tmp_path = tmp_output_path(output_file);

    // Set up Ctrl-C to clean up the temp file
    let tmp_clone = tmp_path.clone();
    let _ = ctrlc::set_handler(move || {
        let _ = std::fs::remove_file(&tmp_clone);
        eprintln!("\nInterrupted. Partial archive removed.");
        std::process::exit(1);
    });

    let result = match format {
        ArchiveFormat::Zip => create_zip::create(paths, &tmp_path, config),
        _ => create_tar::create(paths, &tmp_path, format, config),
    };

    match result {
        Ok(()) => {
            std::fs::rename(&tmp_path, output_file)?;
            if !config.quiet {
                let archive_size = std::fs::metadata(output_file)?.len();
                eprintln!(
                    "Created {} ({})",
                    output_file.display(),
                    output::format_bytes(archive_size)
                );
            }
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            Err(e)
        }
    }
}

/// Extract an archive.
pub fn extract(
    archive_path: &Path,
    output_dir: Option<&Path>,
    specific_files: &[String],
    config: &Config,
) -> Result<()> {
    if !archive_path.exists() {
        return Err(format!("archive not found: {}", archive_path.display()).into());
    }

    let format = resolve_format(Some(archive_path), config)?;
    let dest = output_dir.unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dest)?;

    match format {
        ArchiveFormat::Zip => extract_zip::extract(archive_path, dest, specific_files, config),
        _ => extract_tar::extract(archive_path, dest, format, specific_files, config),
    }
}

/// List contents of an archive.
pub fn list(archive_path: &Path, config: &Config) -> Result<()> {
    if !archive_path.exists() {
        return Err(format!("archive not found: {}", archive_path.display()).into());
    }

    let format = resolve_format(Some(archive_path), config)?;

    match format {
        ArchiveFormat::Zip => extract_zip::list(archive_path, config),
        _ => extract_tar::list(archive_path, format, config),
    }
}

/// Show archive info/statistics.
pub fn info(archive_path: &Path, config: &Config) -> Result<()> {
    if !archive_path.exists() {
        return Err(format!("archive not found: {}", archive_path.display()).into());
    }

    let format = resolve_format(Some(archive_path), config)?;
    let archive_size = std::fs::metadata(archive_path)?.len();

    match format {
        ArchiveFormat::Zip => extract_zip::info(archive_path, archive_size, format, config),
        _ => extract_tar::info(archive_path, archive_size, format, config),
    }
}

/// Verify archive integrity.
pub fn verify(archive_path: &Path, config: &Config) -> Result<()> {
    if !archive_path.exists() {
        return Err(format!("archive not found: {}", archive_path.display()).into());
    }

    let format = resolve_format(Some(archive_path), config)?;

    match format {
        ArchiveFormat::Zip => extract_zip::verify(archive_path, config),
        _ => extract_tar::verify(archive_path, format, config),
    }
}

/// Determine archive format from path/flags/config.
fn resolve_format(path: Option<&Path>, config: &Config) -> Result<ArchiveFormat> {
    // CLI --format flag takes priority
    if let Some(ref choice) = config.format_override {
        return Ok(ArchiveFormat::from_choice(choice));
    }

    // Auto-detect from file extension
    if let Some(p) = path {
        if let Some(fmt) = ArchiveFormat::from_path(p) {
            return Ok(fmt);
        }
    }

    // Fall back to config default
    match config.default_format.as_str() {
        "tar" => Ok(ArchiveFormat::Tar),
        "tar.gz" | "tgz" => Ok(ArchiveFormat::TarGz),
        "tar.zst" | "tar.zstd" => Ok(ArchiveFormat::TarZst),
        "tar.xz" | "txz" => Ok(ArchiveFormat::TarXz),
        "zip" => Ok(ArchiveFormat::Zip),
        other => Err(format!("unknown format: {other}").into()),
    }
}

/// Generate a temporary output path next to the target file.
fn tmp_output_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "archive".to_string());
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{name}.pack_tmp"))
}

/// Check if a path component would escape the destination directory
/// (path traversal protection).
pub fn is_safe_path(path: &Path, dest: &Path) -> bool {
    let full = dest.join(path);
    match full.canonicalize() {
        Ok(resolved) => resolved.starts_with(
            dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf()),
        ),
        // If the file doesn't exist yet, do a component-level check
        Err(_) => !path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir)),
    }
}
