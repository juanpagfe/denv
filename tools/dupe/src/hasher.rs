use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use crate::config::HashChoice;

const CHUNK_SIZE: usize = 64 * 1024; // 64 KB read chunks
const PARTIAL_HASH_SIZE: u64 = 4096; // First 4 KB for partial hash
const HEAD_SAMPLE_SIZE: usize = 4096; // First bytes comparison

/// Read the first N bytes of a file for quick comparison.
pub fn read_head(path: &Path, size: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; size];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

/// Compute a partial hash (first PARTIAL_HASH_SIZE bytes).
pub fn partial_hash(path: &Path, algorithm: HashChoice) -> io::Result<String> {
    hash_range(path, algorithm, 0, Some(PARTIAL_HASH_SIZE))
}

/// Compute the full hash of a file, reading in chunks to avoid loading
/// the entire file into memory.
pub fn full_hash(path: &Path, algorithm: HashChoice) -> io::Result<String> {
    hash_range(path, algorithm, 0, None)
}

/// Hash a byte range of a file.
fn hash_range(
    path: &Path,
    algorithm: HashChoice,
    offset: u64,
    max_bytes: Option<u64>,
) -> io::Result<String> {
    let mut file = File::open(path)?;
    if offset > 0 {
        file.seek(SeekFrom::Start(offset))?;
    }

    match algorithm {
        HashChoice::Blake3 => hash_blake3(&mut file, max_bytes),
        HashChoice::Sha256 => hash_sha256(&mut file, max_bytes),
    }
}

fn hash_blake3(file: &mut File, max_bytes: Option<u64>) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; CHUNK_SIZE];
    let mut remaining = max_bytes.unwrap_or(u64::MAX);

    loop {
        let to_read = (remaining as usize).min(CHUNK_SIZE);
        if to_read == 0 {
            break;
        }
        let n = file.read(&mut buf[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_sha256(file: &mut File, max_bytes: Option<u64>) -> io::Result<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let mut buf = [0u8; CHUNK_SIZE];
    let mut remaining = max_bytes.unwrap_or(u64::MAX);

    loop {
        let to_read = (remaining as usize).min(CHUNK_SIZE);
        if to_read == 0 {
            break;
        }
        let n = file.read(&mut buf[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Default head sample size for quick byte comparison.
pub fn head_sample_size() -> usize {
    HEAD_SAMPLE_SIZE
}
