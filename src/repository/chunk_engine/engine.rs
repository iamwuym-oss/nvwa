// ============================================================================
// engine.rs 閳?ChunkEngine: reads a data stream, produces blocks
// ============================================================================
//
// ChunkEngine is the bridge between data source (Collector) and storage (Repository).
// It reads from any std::io::Read source, splits data into chunks per ChunkPolicy,
// writes blocks to BlockStore, and returns the mapping metadata.
//
// Data flow:
//   Read stream 閳?ChunkEngine 閳?[BlockStore::put_block()] 閳?Block
//                             閳?[return ChunkResult vec]   閳?BlockMap + Catalog
//
// Key design rules:
// - ChunkEngine does NOT know about Backup Jobs, Restore Points, or files
// - ChunkEngine does NOT own the BlockStore reference
// - ChunkResult decouples chunking from indexing

use crate::repository::block_store::block_header::{BlockHeader, Compression};
use crate::repository::block_store::block_id::BlockId;
use crate::repository::block_store::store::{Block, BlockStore};
use crate::repository::chunk_engine::policy::ChunkPolicy;
use crate::repository::error::RepositoryError;
use std::io::Read;

/// Result of processing one chunk through the ChunkEngine.
///
/// This is the bridge between ChunkEngine and BlockMap/Catalog:
/// - logical_offset 閳?BlockMap.insert_mapping()
/// - block_id 閳?referenced in BlockMap
/// - raw_size + stored_size 閳?metadata tracking
#[derive(Debug, Clone)]
pub struct ChunkResult {
    /// Starting offset of this chunk in the logical data stream
    pub logical_offset: u64,
    /// Block identity (SHA-256 of raw data)
    pub block_id: BlockId,
    /// Size of the original uncompressed data
    pub raw_size: u64,
    /// Size stored on disk (after compression, if enabled)
    pub stored_size: u64,
}

/// ChunkEngine 閳?processes data streams into blocks.
///
/// # Usage
/// ```ignore
/// let policy = FixedChunkPolicy::new(262144).unwrap();
/// let engine = ChunkEngine::new(Box::new(policy), true);
/// let results = engine.process(&mut file, &block_store)?;
/// // results now contain mapping data for BlockMap + Catalog
/// ```
pub struct ChunkEngine {
    policy: Box<dyn ChunkPolicy>,
    compression: bool,
}

impl ChunkEngine {
    /// Create a new ChunkEngine.
    ///
    /// # Arguments
    /// * `policy` 閳?ChunkPolicy defining chunk boundaries
    /// * `compression` 閳?If true, blocks are zstd-compressed before storage
    pub fn new(policy: Box<dyn ChunkPolicy>, compression: bool) -> Self {
        ChunkEngine {
            policy,
            compression,
        }
    }

    /// Process a data stream into blocks.
    ///
    /// Reads from `reader` in chunk-sized increments, writes each chunk
    /// as a block to `block_store`, and returns the mapping results.
    ///
    /// # Arguments
    /// * `reader` 閳?Any std::io::Read source (file, pipe, byte buffer)
    /// * `block_store` 閳?Block storage backend
    ///
    /// # Returns
    /// Vec<ChunkResult> 閳?one entry per block, in stream order.
    /// Use these results to populate BlockMap and Catalog.
    ///
    /// # Crash Safety
    /// Each block is written atomically by BlockStore (.tmp 閳?rename).
    /// If processing is interrupted mid-stream, already-written blocks
    /// remain in block-store but are orphan candidates (no BlockMap entry).
    pub fn process<R: Read>(
        &self,
        reader: &mut R,
        block_store: &dyn BlockStore,
    ) -> Result<Vec<ChunkResult>, RepositoryError> {
        let chunk_size = self.policy.chunk_size() as usize;
        let mut results = Vec::new();
        let mut logical_offset: u64 = 0;

        // Reusable buffer to avoid repeated allocations
        let mut buffer = vec![0u8; chunk_size];

        loop {
            // Read up to chunk_size bytes
            let mut bytes_read = 0usize;
            while bytes_read < chunk_size {
                match reader.read(&mut buffer[bytes_read..]) {
                    Ok(0) => break, // EOF
                    Ok(n) => bytes_read += n,
                    Err(e) => {
                        return Err(RepositoryError::General {
                            detail: format!(
                                "ChunkEngine: read error at offset {}: {}",
                                logical_offset, e
                            ),
                        });
                    }
                }
            }

            if bytes_read == 0 {
                break; // End of stream
            }

            let raw_data = &buffer[..bytes_read];

            // Compute block identity from RAW data (Architecture 鎼?.2)
            let block_id = BlockId::from_raw_data(raw_data);

            // Compress data if enabled
            let (stored_data, stored_size) = if self.compression {
                Self::compress(raw_data)?
            } else {
                (raw_data.to_vec(), bytes_read as u64)
            };

            // Create block with appropriate header
            let compression_type = if self.compression {
                Compression::Zstd
            } else {
                Compression::None
            };

            let header = BlockHeader::new(
                compression_type,
                bytes_read as u64, // raw_size = actual bytes read
                stored_size,
            );

            let block = Block {
                header,
                data: stored_data,
            };

            // Write block to store (BlockStore verifies SHA-256 on write)
            let stored_id = block_store.put_block(&block)?;

            // Verify the stored block_id matches our computed one
            // (double-check for data integrity)
            if stored_id != block_id {
                return Err(RepositoryError::General {
                    detail: format!(
                        "ChunkEngine: block_id mismatch after put_block: \
                         expected {}, got {}",
                        block_id, stored_id
                    ),
                });
            }

            results.push(ChunkResult {
                logical_offset,
                block_id,
                raw_size: bytes_read as u64,
                stored_size,
            });

            logical_offset += bytes_read as u64;
        }

        Ok(results)
    }

    /// Compress raw data using zstd
    #[cfg(feature = "compress")]
    fn compress(data: &[u8]) -> Result<(Vec<u8>, u64), RepositoryError> {
        let compressed = zstd::bulk::compress(data, 3).map_err(|e| RepositoryError::General {
            detail: format!("ChunkEngine: zstd compression failed: {}", e),
        })?;
        let stored_size = compressed.len() as u64;
        Ok((compressed, stored_size))
    }

    /// Fallback when compression is not available (should not happen
    /// because `repository` feature implies `compress`)
    #[cfg(not(feature = "compress"))]
    fn compress(data: &[u8]) -> Result<(Vec<u8>, u64), RepositoryError> {
        Err(RepositoryError::General {
            detail: "Compression requested but zstd feature is not enabled".to_string(),
        })
    }

    /// Return the chunk policy
    pub fn policy(&self) -> &dyn ChunkPolicy {
        self.policy.as_ref()
    }

    /// Return whether compression is enabled
    pub fn compression_enabled(&self) -> bool {
        self.compression
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::block_store::store::LocalFsBlockStore;
    use crate::repository::chunk_engine::policy::FixedChunkPolicy;
    use std::io::Cursor;
    use tempfile::TempDir;

    fn setup() -> (LocalFsBlockStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let store = LocalFsBlockStore::new(tmp.path().join("block-store"));
        (store, tmp)
    }

    #[test]
    fn test_chunk_engine_empty_stream() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        let mut empty = Cursor::new(Vec::new());
        let results = engine.process(&mut empty, &store).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_chunk_engine_single_chunk_exact() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        let data = vec![0xABu8; 4096];
        let mut cursor = Cursor::new(data.clone());
        let results = engine.process(&mut cursor, &store).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].logical_offset, 0);
        assert_eq!(results[0].raw_size, 4096);
        assert_eq!(results[0].stored_size, 4096);

        // Verify block_id is computed correctly
        let expected_id = BlockId::from_raw_data(&data);
        assert_eq!(results[0].block_id, expected_id);
    }

    #[test]
    fn test_chunk_engine_single_chunk_partial() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(8192).unwrap(); // large chunk
        let engine = ChunkEngine::new(Box::new(policy), false);

        let data = vec![0xCDu8; 100]; // smaller than chunk size
        let mut cursor = Cursor::new(data.clone());
        let results = engine.process(&mut cursor, &store).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].raw_size, 100);
    }

    #[test]
    fn test_chunk_engine_multiple_chunks() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        // 3 full chunks of 4096 bytes each = 12288, plus tail
        let data = vec![0xEFu8; 15000];
        let block_id_expected = BlockId::from_raw_data(&data[0..4096]);

        let mut cursor = Cursor::new(data);
        let results = engine.process(&mut cursor, &store).unwrap();

        assert_eq!(results.len(), 4); // 4096 + 4096 + 4096 + 2712
        assert_eq!(results[0].raw_size, 4096);
        assert_eq!(results[0].block_id, block_id_expected);
        assert_eq!(results[1].logical_offset, 4096);
        assert_eq!(results[2].logical_offset, 8192);
        assert_eq!(results[3].logical_offset, 12288);
        assert_eq!(results[3].raw_size, 2712);
    }

    #[test]
    fn test_chunk_engine_blocks_actually_stored() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        let data = b"Hello, Nuwa Backup Repository Engine!";
        let mut cursor = Cursor::new(data.to_vec());
        let results = engine.process(&mut cursor, &store).unwrap();

        // Each block should be retrievable
        for result in &results {
            let retrieved = store.get_block(&result.block_id).unwrap();
            assert_eq!(retrieved.header.raw_size, result.raw_size);
            assert_eq!(retrieved.header.raw_size, result.raw_size);
        }
    }

    #[test]
    fn test_chunk_engine_with_compression() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), true);

        // Use repetitive data that compresses well
        let data = vec![0x42u8; 4096];
        let mut cursor = Cursor::new(data);
        let results = engine.process(&mut cursor, &store).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].raw_size, 4096);
        // Compressed size should be smaller than raw size for repetitive data
        assert!(results[0].stored_size < results[0].raw_size);

        // Verify we can retrieve and verify the compressed block
        let retrieved = store.get_block(&results[0].block_id).unwrap();
        assert!(retrieved.header.compression == Compression::Zstd);
    }

    #[test]
    fn test_chunk_engine_many_small_chunks() {
        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        let data = vec![0xFFu8; 75000];
        let mut cursor = Cursor::new(data);
        let results = engine.process(&mut cursor, &store).unwrap();

        // 75000 / 4096 = 18 chunks + 1752 byte tail = 19 chunks
        assert_eq!(results.len(), 19);
        assert_eq!(results[0].logical_offset, 0);
        assert_eq!(results[18].logical_offset, 18 * 4096);
        assert_eq!(results[18].raw_size, 75000 - 18 * 4096);
    }

    #[test]
    fn test_chunk_engine_read_error_propagation() {
        // Use a reader that always errors
        struct ErrorReader;

        impl Read for ErrorReader {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("simulated error"))
            }
        }

        let (store, _tmp) = setup();
        let policy = FixedChunkPolicy::new(4096).unwrap();
        let engine = ChunkEngine::new(Box::new(policy), false);

        let result = engine.process(&mut ErrorReader, &store);
        assert!(result.is_err());
    }
}
