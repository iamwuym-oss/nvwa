//! Record type identifiers for the NWB binary format.
//!
//! Each [`RecordType`] variant carries an explicit 16-bit discriminant that
//! appears as the record header type field on wire.  The compiler rejects any
//! duplicate discriminant at compile time.
//!
//! # Classification
//!
//! | Range     | Category            |
//! |-----------|---------------------|
//! | 0x00xx    | Container / support |
//! | 0x01xx    | Chunk / extent      |
//! | 0x02xx    | Catalog / index     |
//! | 0x03xx    | Error record        |
//! | 0x04xx    | Disk / filesystem   |
//! | 0x05xx    | Cryptography        |

use core::fmt;

/// 16-bit record type discriminator.
///
/// Every physical record in an NWB archive begins with a record header that
/// contains one of these values.  The `Invalid` variant (0x0000) is reserved
/// and must never appear in a valid stream.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u16)]
#[allow(clippy::upper_case_acronyms)]
pub enum RecordType {
    /// Invalid/zero sentinel.
    Invalid = 0x0000,
    /// Volume manifest.
    Manifest = 0x0001,
    /// Volume set manifest.
    VolumeSetManifest = 0x0002,
    /// Volume footer.
    VolumeFooter = 0x0003,
    /// Tombstone.
    Tombstone = 0x0004,
    /// Compressed/encrypted data chunk.
    DataChunk = 0x0101,
    /// Zero-byte run (sparse region descriptor).
    ZeroRun = 0x0102,
    /// Hole run (unallocated region descriptor).
    HoleRun = 0x0103,
    /// File extent.
    FileExtent = 0x0104,
    /// Disk extent.
    DiskExtent = 0x0105,
    /// Differential operation.
    DiffOperation = 0x0106,
    /// Catalog page.
    CatalogPage = 0x0201,
    /// Chunk index page.
    ChunkIndexPage = 0x0202,
    /// Error record.
    ErrorRecord = 0x0301,
    /// File entry.
    FileEntry = 0x0401,
    /// Partition layout.
    PartitionLayout = 0x0402,
    /// BMR artifact.
    BmrArtifact = 0x0403,
    /// Key slot.
    KeySlot = 0x0501,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Invalid => "Invalid",
                Self::Manifest => "Manifest",
                Self::VolumeSetManifest => "VolumeSetManifest",
                Self::VolumeFooter => "VolumeFooter",
                Self::Tombstone => "Tombstone",
                Self::DataChunk => "DataChunk",
                Self::ZeroRun => "ZeroRun",
                Self::HoleRun => "HoleRun",
                Self::FileExtent => "FileExtent",
                Self::DiskExtent => "DiskExtent",
                Self::DiffOperation => "DiffOperation",
                Self::CatalogPage => "CatalogPage",
                Self::ChunkIndexPage => "ChunkIndexPage",
                Self::ErrorRecord => "ErrorRecord",
                Self::FileEntry => "FileEntry",
                Self::PartitionLayout => "PartitionLayout",
                Self::BmrArtifact => "BmrArtifact",
                Self::KeySlot => "KeySlot",
            }
        )
    }
}
