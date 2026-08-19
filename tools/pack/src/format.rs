use std::path::Path;

use crate::cli::FormatChoice;

/// Supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveFormat {
    Tar,
    TarGz,
    TarZst,
    TarXz,
    Zip,
}

impl ArchiveFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?.to_lowercase();

        if name.ends_with(".tar.zst") || name.ends_with(".tar.zstd") {
            Some(Self::TarZst)
        } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            Some(Self::TarXz)
        } else if name.ends_with(".tar") {
            Some(Self::Tar)
        } else if name.ends_with(".zip") {
            Some(Self::Zip)
        } else {
            None
        }
    }

    /// Convert from CLI format choice.
    pub fn from_choice(choice: &FormatChoice) -> Self {
        match choice {
            FormatChoice::Tar => Self::Tar,
            FormatChoice::TarGz => Self::TarGz,
            FormatChoice::TarZst => Self::TarZst,
            FormatChoice::TarXz => Self::TarXz,
            FormatChoice::Zip => Self::Zip,
        }
    }

    /// Display name for the format.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::TarZst => "tar.zst",
            Self::TarXz => "tar.xz",
            Self::Zip => "zip",
        }
    }

    /// Default compression level for this format.
    pub fn default_compression(&self) -> i32 {
        match self {
            Self::TarGz => 6,
            Self::TarZst => 3,
            Self::TarXz => 6,
            _ => 0,
        }
    }
}
