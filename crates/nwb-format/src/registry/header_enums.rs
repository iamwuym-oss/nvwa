//! Header enumeration types for the NWB volume header.
//!
//! These enums appear in the fixed-size volume header fields.

use core::fmt;

/// Type of backup contained in the archive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum BackupKind {
    /// Full backup: all selected data.
    Full = 1,
    /// Differential backup: changes since last full backup.
    Differential = 2,
}

impl fmt::Display for BackupKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "Full"),
            Self::Differential => write!(f, "Differential"),
        }
    }
}

/// Originating platform hint carried in the volume header.
///
/// This is a hint only; the archive format is platform-independent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PlatformHint {
    /// Unknown or unspecified platform.
    Unknown = 0,
    /// Microsoft Windows.
    Windows = 1,
    /// Linux.
    Linux = 2,
}

impl fmt::Display for PlatformHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Windows => write!(f, "Windows"),
            Self::Linux => write!(f, "Linux"),
        }
    }
}
