// ============================================================================
// block_id.rs — Block identity: SHA-256(raw data)
// ============================================================================
//
// Block identity is ALWAYS computed from RAW (uncompressed) data.
// This ensures that changing compression algorithms does not change block identity.
// See Architecture v1.0 §5.2 for rationale.

use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

/// Size of a SHA-256 hash in bytes
pub const HASH_SIZE: usize = 32;

/// BlockId wraps a SHA-256 hash that uniquely identifies a block.
///
/// block_id = SHA-256(raw_chunk_data)
///
/// Rules:
/// - Never computed from compressed data
/// - Immutable once stored
/// - Filename in block-store is the hex representation
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId([u8; HASH_SIZE]);

impl BlockId {
    /// Compute BlockId from raw data using SHA-256
    pub fn from_raw_data(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut bytes = [0u8; HASH_SIZE];
        bytes.copy_from_slice(&result);
        BlockId(bytes)
    }

    /// Create a BlockId from raw bytes (must be exactly 32 bytes)
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != HASH_SIZE {
            return None;
        }
        let mut arr = [0u8; HASH_SIZE];
        arr.copy_from_slice(bytes);
        Some(BlockId(arr))
    }

    /// Return the raw 32 bytes
    pub fn as_bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }

    /// Return hex string representation
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Return the first two hex characters (for first-level directory)
    pub fn dir_prefix_1(&self) -> String {
        let hex = self.to_hex();
        hex[0..2].to_string()
    }

    /// Return hex characters 2-4 (for second-level directory)
    pub fn dir_prefix_2(&self) -> String {
        let hex = self.to_hex();
        hex[2..4].to_string()
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockId({})", self.to_hex())
    }
}

impl FromStr for BlockId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {}", e))?;
        Self::from_bytes(&bytes).ok_or_else(|| "Expected 32 bytes (64 hex chars)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id_from_raw_data() {
        let data = b"Hello, Nuwa Backup!";
        let id = BlockId::from_raw_data(data);
        // SHA-256 is deterministic
        let id2 = BlockId::from_raw_data(data);
        assert_eq!(id, id2);
        assert_eq!(id.to_hex().len(), 64);
    }

    #[test]
    fn test_block_id_different_data() {
        let id1 = BlockId::from_raw_data(b"data1");
        let id2 = BlockId::from_raw_data(b"data2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_block_id_roundtrip_hex() {
        let data = b"some test content for block id roundtrip";
        let id = BlockId::from_raw_data(data);
        let hex = id.to_hex();
        let parsed: BlockId = hex.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_block_id_from_bytes() {
        let arr = [0xABu8; HASH_SIZE];
        let id = BlockId::from_bytes(&arr).unwrap();
        assert_eq!(id.as_bytes(), &arr);
    }

    #[test]
    fn test_block_id_wrong_length() {
        assert!(BlockId::from_bytes(&[0; 16]).is_none());
    }

    #[test]
    fn test_dir_prefixes() {
        let data = b"prefix test data";
        let id = BlockId::from_raw_data(data);
        let hex = id.to_hex();
        assert_eq!(id.dir_prefix_1(), &hex[0..2]);
        assert_eq!(id.dir_prefix_2(), &hex[2..4]);
    }
}
