// ============================================================================
// policy.rs — ChunkPolicy trait and FixedChunkPolicy implementation
// ============================================================================
//
// ChunkPolicy determines how a data stream is split into blocks.
// Phase S implements FixedChunkPolicy only. CDC (content-defined chunking)
// is reserved for future optimization.
//
// Key design rules:
// - ChunkPolicy does NOT know about storage, hashing, or compression
// - It only answers: "where does the next chunk boundary fall?"
// - FixedChunkPolicy always splits at fixed byte boundaries

use crate::repository::error::RepositoryError;

/// ChunkPolicy trait — determines chunk boundaries for a data stream.
///
/// # Contract
/// - chunk_size() returns the nominal chunk size in bytes
/// - next_boundary() may return None if the chunk is not yet complete
///   (reserved for future CDC use)
pub trait ChunkPolicy: Send + Sync {
    /// Return the nominal chunk size in bytes
    fn chunk_size(&self) -> u32;

    /// Given accumulated `data` (starting at `stream_position`),
    /// return the byte offset within `data` where the next chunk boundary falls,
    /// or None if no boundary exists yet.
    ///
    /// For FixedChunkPolicy, this returns chunk_size when data.len() >= chunk_size.
    fn next_boundary(&self, stream_position: u64, data: &[u8]) -> Option<usize>;
}

/// Fixed-size chunk policy — splits data at fixed byte boundaries.
///
/// Phase S uses fixed-size chunks (default 256KB, from repo metadata).
/// This is the simplest and most predictable chunking strategy.
///
/// # Rationale for fixed-size in Phase S
/// - Predictable block count estimation before backup runs
/// - Simple implementation with zero CPU overhead
/// - CDC (content-defined chunking) can be layered later without format break
///   because block-map decouples logical_offset from storage
#[derive(Debug, Clone)]
pub struct FixedChunkPolicy {
    chunk_size: u32,
}

impl FixedChunkPolicy {
    /// Create a new FixedChunkPolicy with the given chunk size.
    ///
    /// # Errors
    /// Returns `RepositoryError::General` if chunk_size is not between
    /// 4KB (4096) and 4MB (4194304).
    pub fn new(chunk_size: u32) -> Result<Self, RepositoryError> {
        if !(4096..=4_194_304).contains(&chunk_size) {
            return Err(RepositoryError::General {
                detail: format!(
                    "FixedChunkPolicy: chunk_size must be between 4KB and 4MB, got {}",
                    chunk_size
                ),
            });
        }
        Ok(FixedChunkPolicy { chunk_size })
    }

    /// Return the configured chunk size
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }
}

impl ChunkPolicy for FixedChunkPolicy {
    fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// For FixedChunkPolicy, a boundary exists when accumulated data
    /// reaches or exceeds chunk_size.
    fn next_boundary(&self, _stream_position: u64, data: &[u8]) -> Option<usize> {
        if data.len() >= self.chunk_size as usize {
            Some(self.chunk_size as usize)
        } else {
            // Not enough data for a full chunk; caller should accumulate more
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_policy_chunk_size() {
        let policy = FixedChunkPolicy::new(262144).unwrap();
        assert_eq!(policy.chunk_size(), 262144);
    }

    #[test]
    fn test_fixed_policy_boundary() {
        let policy = FixedChunkPolicy::new(4096).unwrap();
        // Less than chunk size — no boundary
        assert!(policy.next_boundary(0, &[0u8; 1000]).is_none());
        // Exactly chunk size — boundary at chunk_size
        assert_eq!(policy.next_boundary(0, &[0u8; 4096]), Some(4096));
        // More than chunk size — boundary at chunk_size
        assert_eq!(policy.next_boundary(0, &[0u8; 8192]), Some(4096));
    }

    #[test]
    fn test_fixed_policy_default_chunk_size() {
        let policy = FixedChunkPolicy::new(262144).unwrap();
        assert_eq!(policy.chunk_size(), 262144);
    }

    #[test]
    fn test_invalid_chunk_size_too_small() {
        let result = FixedChunkPolicy::new(100);
        assert!(result.is_err(), "chunk_size too small should return Err");
    }

    #[test]
    fn test_invalid_chunk_size_too_large() {
        let result = FixedChunkPolicy::new(10_000_000);
        assert!(result.is_err(), "chunk_size too large should return Err");
    }

    #[test]
    fn test_stream_position_unchanged() {
        // Verify stream_position doesn't affect fixed-boundary decision
        let policy = FixedChunkPolicy::new(4096).unwrap();
        assert_eq!(policy.next_boundary(999999, &[0u8; 4096]), Some(4096));
        assert!(policy.next_boundary(999999, &[0u8; 1000]).is_none());
    }
}
