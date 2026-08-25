use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

use chrono::{DateTime, Local};

/// A single clipboard history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The copied text content.
    pub content: String,
    /// When the entry was copied.
    pub timestamp: DateTime<Local>,
    /// Number of bytes in the content.
    pub bytes: usize,
    /// Number of lines in the content.
    pub lines: usize,
}

/// Clipboard history storage backed by a JSON file.
pub struct History {
    entries: Vec<Entry>,
    path: std::path::PathBuf,
    max_size: usize,
}

impl History {
    /// Load history from disk, or create empty if it doesn't exist.
    pub fn load(path: &Path, max_size: usize) -> io::Result<Self> {
        let entries = if path.exists() {
            let data = fs::read_to_string(path)?;
            serde_json::from_str(&data).unwrap_or_else(|e| {
                eprintln!("warning: failed to parse history file: {e}");
                Vec::new()
            })
        } else {
            Vec::new()
        };

        Ok(Self {
            entries,
            path: path.to_path_buf(),
            max_size,
        })
    }

    /// Add a new entry to history and persist to disk.
    pub fn add(&mut self, content: String) -> io::Result<()> {
        // Skip empty content
        if content.is_empty() {
            return Ok(());
        }

        // Remove duplicates of the same content
        self.entries.retain(|e| e.content != content);

        let bytes = content.len();
        let lines = content.lines().count();

        let entry = Entry {
            content,
            timestamp: Local::now(),
            bytes,
            lines,
        };

        self.entries.insert(0, entry);

        // Truncate to max size
        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }

        self.save()
    }

    /// Get all entries (most recent first).
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Clear all history entries and persist.
    pub fn clear(&mut self) -> io::Result<()> {
        self.entries.clear();
        self.save()
    }

    /// Persist history to disk.
    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.path, data)
    }
}
