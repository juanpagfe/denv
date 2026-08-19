use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::Config;
use crate::hasher;
use crate::output;
use crate::scanner::{DuplicateGroup, FileEntry};

/// Run interactive clean mode: scan for duplicates, then let the user choose
/// which copies to delete.
pub fn run_clean(
    paths: &[PathBuf],
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let groups = scan_for_groups(paths, config)?;

    if groups.is_empty() {
        if !config.quiet {
            eprintln!("No duplicates found.");
        }
        return Ok(());
    }

    let mut total_freed: u64 = 0;
    let mut total_deleted: u64 = 0;

    for (i, group) in groups.iter().enumerate() {
        println!();
        println!(
            "Duplicate group #{} — {} each, {} files",
            i + 1,
            output::format_bytes(group.files[0].size),
            group.files.len()
        );
        println!();

        for (j, file) in group.files.iter().enumerate() {
            println!("  [{}] {}", j + 1, file.path.display());
        }

        println!();
        println!("  [k] Keep all");
        println!("  [q] Quit");
        println!();
        print!("  Keep which file? [1-{}, k, q]: ", group.files.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "q" => {
                println!("Aborted.");
                break;
            }
            "k" | "" => {
                println!("  Keeping all.");
                continue;
            }
            _ => {
                if let Ok(keep_idx) = input.parse::<usize>() {
                    if keep_idx >= 1 && keep_idx <= group.files.len() {
                        // Delete all except the chosen one
                        for (j, file) in group.files.iter().enumerate() {
                            if j + 1 == keep_idx {
                                continue;
                            }

                            // Verify file hasn't changed before deleting
                            if verify_unchanged(file, &group.hash, config) {
                                match fs::remove_file(&file.path) {
                                    Ok(()) => {
                                        println!("  Deleted: {}", file.path.display());
                                        total_freed += file.size;
                                        total_deleted += 1;
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "  Error deleting {}: {}",
                                            file.path.display(),
                                            e
                                        );
                                    }
                                }
                            } else {
                                eprintln!(
                                    "  Skipping {} (file changed since scan)",
                                    file.path.display()
                                );
                            }
                        }
                    } else {
                        println!("  Invalid selection, keeping all.");
                    }
                } else {
                    println!("  Invalid input, keeping all.");
                }
            }
        }
    }

    if total_deleted > 0 {
        println!();
        println!(
            "Deleted {} files, freed {}",
            total_deleted,
            output::format_bytes(total_freed)
        );
    }

    Ok(())
}

/// Run the scan pipeline and return duplicate groups with their hashes.
fn scan_for_groups(
    paths: &[PathBuf],
    config: &Config,
) -> Result<Vec<DuplicateGroup>, Box<dyn std::error::Error>> {
    // We reuse the scanner pipeline but need the groups back.
    // For now, call run_scan internals directly.
    // This is a simplified version — the full scan pipeline is in scanner.rs
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.workers)
        .build_global()
        .ok();

    if !config.quiet {
        eprintln!("Scanning for duplicates...");
    }

    let files = walk_and_collect(paths, config)?;

    if files.is_empty() {
        return Ok(vec![]);
    }

    // Progressive pipeline (same as scanner)
    let size_groups = group_by_key(files, |f| f.size);
    let candidates = flatten_duplicates(size_groups);

    if candidates.is_empty() {
        return Ok(vec![]);
    }

    let head_groups = group_by_key(candidates, |f| {
        hasher::read_head(&f.path, hasher::head_sample_size())
            .unwrap_or_default()
    });
    let candidates = flatten_duplicates(head_groups);

    if candidates.is_empty() {
        return Ok(vec![]);
    }

    let partial_groups = group_by_key(candidates, |f| {
        hasher::partial_hash(&f.path, config.hash_algorithm)
            .unwrap_or_default()
    });
    let candidates = flatten_duplicates(partial_groups);

    if candidates.is_empty() {
        return Ok(vec![]);
    }

    // Full hash
    let full_groups = group_by_key_with_hash(candidates, config);

    let groups: Vec<DuplicateGroup> = full_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| DuplicateGroup { hash, files })
        .collect();

    Ok(groups)
}

fn walk_and_collect(
    paths: &[PathBuf],
    config: &Config,
) -> Result<Vec<FileEntry>, io::Error> {
    use std::sync::{Arc, Mutex};
    let files = Arc::new(Mutex::new(Vec::new()));
    for path in paths {
        walk_recursive(path, config, &files, 0)?;
    }
    Ok(Arc::try_unwrap(files)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap())
}

fn walk_recursive(
    dir: &std::path::Path,
    config: &Config,
    files: &std::sync::Arc<std::sync::Mutex<Vec<FileEntry>>>,
    depth: usize,
) -> Result<(), io::Error> {
    if let Some(max) = config.max_depth {
        if depth > max {
            return Ok(());
        }
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = if config.follow_symlinks {
            std::fs::metadata(&path)
        } else {
            std::fs::symlink_metadata(&path)
        };
        let meta = match meta {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_symlink() && !config.follow_symlinks {
            continue;
        }
        if meta.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if config.exclude_dirs.iter().any(|d| d == &name) {
                continue;
            }
            walk_recursive(&path, config, files, depth + 1)?;
            continue;
        }
        if !meta.is_file() || meta.len() == 0 {
            continue;
        }
        let size = meta.len();
        if let Some(min) = config.min_size {
            if size < min { continue; }
        }
        if let Some(max) = config.max_size {
            if size > max { continue; }
        }
        files.lock().unwrap().push(FileEntry { path, size });
    }
    Ok(())
}

fn group_by_key<K: Eq + std::hash::Hash>(
    files: Vec<FileEntry>,
    key_fn: impl Fn(&FileEntry) -> K,
) -> std::collections::HashMap<K, Vec<FileEntry>> {
    let mut groups = std::collections::HashMap::new();
    for file in files {
        let key = key_fn(&file);
        groups.entry(key).or_insert_with(Vec::new).push(file);
    }
    groups
}

fn group_by_key_with_hash(
    files: Vec<FileEntry>,
    config: &Config,
) -> std::collections::HashMap<String, Vec<FileEntry>> {
    let mut groups = std::collections::HashMap::new();
    for file in files {
        let hash = hasher::full_hash(&file.path, config.hash_algorithm)
            .unwrap_or_default();
        groups.entry(hash).or_insert_with(Vec::new).push(file);
    }
    groups
}

fn flatten_duplicates<K>(groups: std::collections::HashMap<K, Vec<FileEntry>>) -> Vec<FileEntry> {
    groups
        .into_iter()
        .filter(|(_, g)| g.len() > 1)
        .flat_map(|(_, g)| g)
        .collect()
}

/// Verify a file still has the same hash (guard against race conditions).
fn verify_unchanged(file: &FileEntry, expected_hash: &str, config: &Config) -> bool {
    match hasher::full_hash(&file.path, config.hash_algorithm) {
        Ok(hash) => hash == expected_hash,
        Err(_) => false,
    }
}

