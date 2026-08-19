use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::config::Config;
use crate::hasher;
use crate::output::{self, ScanStats};

/// A file entry discovered during scanning.
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

/// A group of duplicate files sharing the same content.
pub struct DuplicateGroup {
    pub hash: String,
    pub files: Vec<FileEntry>,
}

/// Run the full duplicate scan pipeline and display results.
pub fn run_scan(paths: &[PathBuf], config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let cancelled = Arc::new(AtomicBool::new(false));

    // Set up Ctrl-C handler
    let cancel_flag = cancelled.clone();
    ctrlc_setup(cancel_flag);

    // Configure rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.workers)
        .build_global()
        .ok(); // Ignore error if already initialized

    // Phase 1: Walk directories and collect files
    if !config.quiet {
        eprintln!("Scanning directories...");
    }
    let files = walk_directories(paths, config)?;
    let total_files = files.len() as u64;

    if cancelled.load(Ordering::Relaxed) {
        return Ok(());
    }

    if !config.quiet {
        eprintln!("Found {} files", total_files);
    }

    // Phase 2: Group by size (files with unique sizes can't be duplicates)
    let size_groups = group_by_size(files);
    let candidates: Vec<FileEntry> = size_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .flat_map(|(_, group)| group)
        .collect();

    if candidates.is_empty() {
        let stats = ScanStats {
            files_scanned: total_files,
            duplicate_groups: 0,
            duplicate_files: 0,
            wasted_space: 0,
            elapsed: start.elapsed(),
        };
        output::display_results(&[], &stats, config)?;
        return Ok(());
    }

    if !config.quiet && config.verbose {
        eprintln!(
            "{} files have non-unique sizes (potential duplicates)",
            candidates.len()
        );
    }

    // Phase 3: Compare head bytes to quickly eliminate non-duplicates
    let head_groups = group_by_head(&candidates, config, &cancelled)?;
    let candidates: Vec<FileEntry> = head_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .flat_map(|(_, group)| group)
        .collect();

    if cancelled.load(Ordering::Relaxed) {
        return Ok(());
    }

    if candidates.is_empty() {
        let stats = ScanStats {
            files_scanned: total_files,
            duplicate_groups: 0,
            duplicate_files: 0,
            wasted_space: 0,
            elapsed: start.elapsed(),
        };
        output::display_results(&[], &stats, config)?;
        return Ok(());
    }

    // Phase 4: Partial hash
    let partial_groups = group_by_hash(&candidates, config, &cancelled, true)?;
    let candidates: Vec<FileEntry> = partial_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .flat_map(|(_, group)| group)
        .collect();

    if cancelled.load(Ordering::Relaxed) {
        return Ok(());
    }

    // Phase 5: Full hash (only for remaining candidates)
    let progress = if !config.quiet {
        let pb = ProgressBar::new(candidates.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} hashing")
                .unwrap()
                .progress_chars("█▓░"),
        );
        Some(pb)
    } else {
        None
    };

    let full_groups = group_by_hash_with_progress(
        &candidates,
        config,
        &cancelled,
        false,
        progress.as_ref(),
    )?;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Build final duplicate groups
    let mut groups: Vec<DuplicateGroup> = full_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .map(|(hash, files)| DuplicateGroup { hash, files })
        .collect();

    // Sort groups by wasted space (largest first)
    groups.sort_by(|a, b| {
        let waste_a: u64 = a.files.iter().skip(1).map(|f| f.size).sum();
        let waste_b: u64 = b.files.iter().skip(1).map(|f| f.size).sum();
        waste_b.cmp(&waste_a)
    });

    let duplicate_files: u64 = groups.iter().map(|g| g.files.len() as u64 - 1).sum();
    let wasted_space: u64 = groups
        .iter()
        .map(|g| {
            let per_file = g.files.first().map(|f| f.size).unwrap_or(0);
            per_file * (g.files.len() as u64 - 1)
        })
        .sum();

    let stats = ScanStats {
        files_scanned: total_files,
        duplicate_groups: groups.len() as u64,
        duplicate_files,
        wasted_space,
        elapsed: start.elapsed(),
    };

    output::display_results(&groups, &stats, config)?;
    Ok(())
}

/// Recursively walk directories collecting file entries, applying filters.
fn walk_directories(
    paths: &[PathBuf],
    config: &Config,
) -> Result<Vec<FileEntry>, io::Error> {
    let files = Arc::new(Mutex::new(Vec::new()));

    for path in paths {
        walk_dir_recursive(path, config, &files, 0)?;
    }

    let result = Arc::try_unwrap(files)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap();
    Ok(result)
}

fn walk_dir_recursive(
    dir: &Path,
    config: &Config,
    files: &Arc<Mutex<Vec<FileEntry>>>,
    depth: usize,
) -> Result<(), io::Error> {
    if let Some(max_depth) = config.max_depth {
        if depth > max_depth {
            return Ok(());
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            if config.verbose {
                eprintln!("warning: cannot read {}: {}", dir.display(), e);
            }
            return Ok(());
        }
    };

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
        let metadata = if config.follow_symlinks {
            fs::metadata(&path)
        } else {
            fs::symlink_metadata(&path)
        };

        let metadata = match metadata {
            Ok(m) => m,
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: cannot stat {}: {}", path.display(), e);
                }
                continue;
            }
        };

        // Skip symlinks unless follow_symlinks is set
        if metadata.is_symlink() && !config.follow_symlinks {
            continue;
        }

        if metadata.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Check exclude_dirs
            if config.exclude_dirs.iter().any(|d| d == &dir_name) {
                continue;
            }

            walk_dir_recursive(&path, config, files, depth + 1)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let size = metadata.len();

        // Skip empty files
        if size == 0 {
            continue;
        }

        // Apply size filters
        if let Some(min) = config.min_size {
            if size < min {
                continue;
            }
        }
        if let Some(max) = config.max_size {
            if size > max {
                continue;
            }
        }

        // Apply include/exclude patterns
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !config.include_patterns.is_empty()
            && !config
                .include_patterns
                .iter()
                .any(|p| matches_glob(p, &file_name))
        {
            continue;
        }
        if config
            .exclude_patterns
            .iter()
            .any(|p| matches_glob(p, &file_name))
        {
            continue;
        }

        files.lock().unwrap().push(FileEntry { path, size });
    }

    Ok(())
}

/// Simple glob matching supporting * and ? wildcards.
fn matches_glob(pattern: &str, name: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let nam: Vec<char> = name.chars().collect();
    glob_match(&pat, &nam, 0, 0)
}

fn glob_match(pat: &[char], name: &[char], pi: usize, ni: usize) -> bool {
    if pi == pat.len() && ni == name.len() {
        return true;
    }
    if pi == pat.len() {
        return false;
    }

    match pat[pi] {
        '*' => {
            // * matches zero or more characters
            for i in ni..=name.len() {
                if glob_match(pat, name, pi + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ni < name.len() {
                glob_match(pat, name, pi + 1, ni + 1)
            } else {
                false
            }
        }
        c => {
            if ni < name.len() && name[ni] == c {
                glob_match(pat, name, pi + 1, ni + 1)
            } else {
                false
            }
        }
    }
}

/// Group files by size.
fn group_by_size(files: Vec<FileEntry>) -> HashMap<u64, Vec<FileEntry>> {
    let mut groups: HashMap<u64, Vec<FileEntry>> = HashMap::new();
    for file in files {
        groups.entry(file.size).or_default().push(file);
    }
    groups
}

/// Group files by head bytes for quick elimination.
fn group_by_head(
    files: &[FileEntry],
    config: &Config,
    cancelled: &Arc<AtomicBool>,
) -> Result<HashMap<(u64, Vec<u8>), Vec<FileEntry>>, io::Error> {
    let groups: Arc<Mutex<HashMap<(u64, Vec<u8>), Vec<FileEntry>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    files.par_iter().for_each(|file| {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }
        match hasher::read_head(&file.path, hasher::head_sample_size()) {
            Ok(head) => {
                let key = (file.size, head);
                groups.lock().unwrap().entry(key).or_default().push(file.clone());
            }
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: cannot read {}: {}", file.path.display(), e);
                }
            }
        }
    });

    let result = Arc::try_unwrap(groups)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap();
    Ok(result)
}

/// Group files by hash (partial or full).
fn group_by_hash(
    files: &[FileEntry],
    config: &Config,
    cancelled: &Arc<AtomicBool>,
    partial: bool,
) -> Result<HashMap<String, Vec<FileEntry>>, io::Error> {
    group_by_hash_with_progress(files, config, cancelled, partial, None)
}

fn group_by_hash_with_progress(
    files: &[FileEntry],
    config: &Config,
    cancelled: &Arc<AtomicBool>,
    partial: bool,
    progress: Option<&ProgressBar>,
) -> Result<HashMap<String, Vec<FileEntry>>, io::Error> {
    let groups: Arc<Mutex<HashMap<String, Vec<FileEntry>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    files.par_iter().for_each(|file| {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }

        let hash_result = if partial {
            hasher::partial_hash(&file.path, config.hash_algorithm)
        } else {
            hasher::full_hash(&file.path, config.hash_algorithm)
        };

        match hash_result {
            Ok(hash) => {
                groups.lock().unwrap().entry(hash).or_default().push(file.clone());
            }
            Err(e) => {
                if config.verbose {
                    eprintln!("warning: cannot hash {}: {}", file.path.display(), e);
                }
            }
        }

        if let Some(pb) = progress {
            pb.inc(1);
        }
    });

    let result = Arc::try_unwrap(groups)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap();
    Ok(result)
}

fn ctrlc_setup(flag: Arc<AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        flag.store(true, Ordering::Relaxed);
        eprintln!("\nInterrupted. Cleaning up...");
    });
}
