//! Error identifiers for the NWB binary format.
//!
//! Each [`ErrorId`] variant carries an explicit 16-bit discriminant that
//! uniquely identifies a class of format-level error.  These identifiers
//! appear in error records within the archive and in diagnostic output.
//!
//! # Classification
//!
//! | Range     | Category              |
//! |-----------|-----------------------|
//! | 0x00xx    | Generic / structural  |
//! | 0x01xx    | I/O / volume          |
//! | 0x02xx    | Encryption            |
//! | 0x03xx    | Catalog               |

use core::fmt;

/// 16-bit error identifier.
///
/// Every format-level error in an NWB archive is classified by one of
/// these discriminants.  The `Invalid` variant (0x0000) is reserved and
/// must not appear in valid diagnostic output.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u16)]
pub enum ErrorId {
    /// Invalid/zero sentinel, must not appear in valid output.
    Invalid = 0x0000,
    /// Data integrity verification failed.
    ChecksumMismatch = 0x0001,
    /// Unsupported or unrecognized format version.
    FormatVersion = 0x0002,
    /// Compression or decompression error.
    Compression = 0x0003,
    /// Segment data is corrupt or truncated.
    SegmentCorrupt = 0x0004,
    /// Input/output operation failed.
    IoError = 0x0100,
    /// Required volume is not available.
    VolumeMissing = 0x0101,
    /// Encryption authentication or decryption failure.
    EncryptionAuth = 0x0200,
    /// Catalog data is corrupt or inconsistent.
    CatalogCorrupt = 0x0300,
}

impl fmt::Display for ErrorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Invalid => "Invalid",
                Self::ChecksumMismatch => "ChecksumMismatch",
                Self::FormatVersion => "FormatVersion",
                Self::Compression => "Compression",
                Self::SegmentCorrupt => "SegmentCorrupt",
                Self::IoError => "IoError",
                Self::VolumeMissing => "VolumeMissing",
                Self::EncryptionAuth => "EncryptionAuth",
                Self::CatalogCorrupt => "CatalogCorrupt",
            }
        )
    }
}
