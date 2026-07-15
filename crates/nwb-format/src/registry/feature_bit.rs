//! Feature bit constants for the NWB format capability bitmask.
//!
//! Each constant defines a single bit position within a 64-bit feature mask
//! carried in the volume header.  A compile-time `const` block verifies that
//! no two constants share the same bit position.
//!
//! # Allocated bits (0–4)
//!
//! | Bit | Name               | Description                    |
//! |----:|--------------------|--------------------------------|
//! |   0 | CompressionZstd    | Zstandard compression          |
//! |   1 | EncryptionAes256Gcm | AES-256-GCM encryption        |
//! |   2 | VolumeSet          | Multi-volume set support       |
//! |   3 | Checkpoint         | Checkpoint/resume support      |
//! |   4 | BmrMetadata        | Bare-metal recovery metadata   |
//! | 5–63 | —                 | Reserved for future allocation |

// --- Bit-position constants (u8) -------------------------------------------

/// Bit position for Zstandard compression support.
pub const COMPRESSION_ZSTD: u8 = 0;

/// Bit position for AES-256-GCM encryption support.
pub const ENCRYPTION_AES256_GCM: u8 = 1;

/// Bit position for multi-volume set support.
pub const VOLUME_SET: u8 = 2;

/// Bit position for checkpoint/resume support.
pub const CHECKPOINT: u8 = 3;

/// Bit position for bare-metal recovery metadata.
pub const BMR_METADATA: u8 = 4;

// --- Compile-time uniqueness check -----------------------------------------
//
// Verifies that every allocated bit position maps to a distinct, valid bit
// in a u64 mask.  A duplicate, zero, or out-of-range position is caught
// before any code is generated.

const _: () = {
    const ALLOCATED: &[u8] = &[
        COMPRESSION_ZSTD,
        ENCRYPTION_AES256_GCM,
        VOLUME_SET,
        CHECKPOINT,
        BMR_METADATA,
    ];

    let mut mask: u64 = 0;
    let mut i: usize = 0;
    while i < ALLOCATED.len() {
        let pos = ALLOCATED[i];
        // Must be within valid bit range.
        assert!((pos as u64) < 64u64);
        let bit = 1u64 << pos;
        // Must not overlap with any previously seen bit.
        assert!(mask & bit == 0u64);
        mask |= bit;
        i += 1;
    }
};
