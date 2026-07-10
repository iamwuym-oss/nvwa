// ============================================================================
// store.rs — BlockStore trait and LocalFsBlockStore implementation
// ============================================================================
//
// BlockStore provides an abstraction over block storage backends.
// Phase S implements LocalFsBlockStore using a two-level directory layout.
// See Architecture v1.0 §5.
//
// Key design rules:
// - Blocks are immutable once written (Write Once, Read Many)
// - Block identity = SHA-256(raw data), verified on read
// - Atomic write: .tmp → rename to prevent partial writes

use crate::repository::block_store::block_header::{BlockHeader, Compression, BLOCK_HEADER_SIZE};
use crate::repository::block_store::block_id::BlockId;
use crate::repository::error::RepositoryError;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A complete block with header and compressed data
#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    /// Compressed (or raw, if no compression) data
    pub data: Vec<u8>,
}

/// BlockStore trait — abstract block storage backend.
///
/// Responsibilities:
/// - Store blocks by block_id (content-addressed)
/// - Retrieve blocks by block_id
/// - Verify block integrity on read (recompute SHA-256 of raw data)
///
/// Does NOT know about Restore Points, chains, or file semantics.
pub trait BlockStore: Send + Sync {
    /// Store a block. The block_id is SHA-256(raw data), computed externally.
    /// Uses atomic write (.tmp → rename) for crash safety.
    fn put_block(&self, block: &Block) -> Result<BlockId, RepositoryError>;

    /// Retrieve a block by its block_id.
    /// On retrieval, SHA-256(raw data) is verified against the block_id.
    fn get_block(&self, id: &BlockId) -> Result<Block, RepositoryError>;

    /// Check if a block exists in the store
    fn exists(&self, id: &BlockId) -> Result<bool, RepositoryError>;

    /// Read and verify a block without returning the full data.
    /// Returns true if the block is intact.
    fn verify_block(&self, id: &BlockId) -> Result<bool, RepositoryError>;

    /// Return the root path of this store (for diagnostics)
    fn root_path(&self) -> &Path;
}

/// Phase S implementation: local filesystem block store.
///
/// Directory layout:
///   block-store/{hash[0:2]}/{hash[2:4]}/{full_hash}.block
///
/// Atomic write:
///   Write to {full_hash}.block.tmp → rename → {full_hash}.block
///   On rename failure, .tmp file is cleaned up.
pub struct LocalFsBlockStore {
    root: PathBuf,
}

impl LocalFsBlockStore {
    /// Create a new LocalFsBlockStore rooted at the given directory.
    /// The directory is created if it does not exist.
    pub fn new(root: PathBuf) -> Self {
        LocalFsBlockStore { root }
    }

    /// Return the filesystem path for a given block id
    fn block_path(&self, id: &BlockId) -> PathBuf {
        let hex = id.to_hex();
        self.root
            .join(id.dir_prefix_1())
            .join(id.dir_prefix_2())
            .join(format!("{}.block", hex))
    }

    /// Return the temporary path used during atomic writes
    fn tmp_path(&self, id: &BlockId) -> PathBuf {
        let hex = id.to_hex();
        self.root
            .join(id.dir_prefix_1())
            .join(id.dir_prefix_2())
            .join(format!("{}.block.tmp", hex))
    }

    /// Ensure the directory for a given block id exists
    fn ensure_dir(&self, id: &BlockId) -> Result<(), RepositoryError> {
        let dir = self.root.join(id.dir_prefix_1()).join(id.dir_prefix_2());
        fs::create_dir_all(&dir)
            .map_err(|e| RepositoryError::io(dir, "Cannot create block store directory", e))
    }
}

impl BlockStore for LocalFsBlockStore {
    fn put_block(&self, block: &Block) -> Result<BlockId, RepositoryError> {
        // Compute block_id from raw data (Architecture v1.0 §5.2)
        // The raw data is obtained by decompressing stored data if needed.
        // For put_block, the caller provides the block which contains compressed data.
        // We recompute SHA-256 of the RAW data to verify correctness.
        let raw_data = if block.header.compression == Compression::None {
            block.data.clone()
        } else {
            // Cannot decompress without knowing the algorithm;
            // the caller should have provided the raw_size for verification.
            // For now, we trust the caller and use the stored data after decompression.
            // In Phase S, we decompress and verify:
            let mut decoder =
                zstd::Decoder::new(&block.data[..]).map_err(|e| RepositoryError::General {
                    detail: format!("zstd decompression failed during put: {}", e),
                })?;
            let mut raw = Vec::with_capacity(block.header.raw_size as usize);
            decoder
                .read_to_end(&mut raw)
                .map_err(|e| RepositoryError::General {
                    detail: format!("Failed to read decompressed data: {}", e),
                })?;
            raw
        };

        let block_id = BlockId::from_raw_data(&raw_data);

        // Create directory structure
        self.ensure_dir(&block_id)?;

        let dest_path = self.block_path(&block_id);
        let tmp_path = self.tmp_path(&block_id);

        // Build the block file: header (64B) + compressed data
        let header_bytes = block.header.encode();
        let mut file_content = Vec::with_capacity(BLOCK_HEADER_SIZE + block.data.len());
        file_content.extend_from_slice(&header_bytes);
        file_content.extend_from_slice(&block.data);

        // === Atomic write: .tmp → rename ===
        // Write to .tmp first, then rename atomically.
        // On failure, clean up .tmp to prevent residue.
        fs::write(&tmp_path, &file_content).map_err(|e| {
            RepositoryError::io(tmp_path.clone(), "Cannot write block temporary file", e)
        })?;

        match fs::rename(&tmp_path, &dest_path) {
            Ok(()) => {}
            Err(e) => {
                let _ = fs::remove_file(&tmp_path);
                return Err(RepositoryError::io(
                    dest_path,
                    "Cannot rename block file to final location",
                    e,
                ));
            }
        }

        Ok(block_id)
    }

    fn get_block(&self, id: &BlockId) -> Result<Block, RepositoryError> {
        let path = self.block_path(id);

        let file_data = fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RepositoryError::block_not_found(&id.to_hex())
            } else {
                RepositoryError::io(path.clone(), "Cannot read block file", e)
            }
        })?;

        if file_data.len() < BLOCK_HEADER_SIZE {
            return Err(RepositoryError::invalid_block_header(
                path,
                format!("File too small: {} bytes", file_data.len()),
            ));
        }

        let header = BlockHeader::decode(&file_data[..BLOCK_HEADER_SIZE])?;
        let data = file_data[BLOCK_HEADER_SIZE..].to_vec();

        if data.len() != header.stored_size as usize {
            return Err(RepositoryError::BlockStoreCorrupted {
                root: self.root.clone(),
                detail: format!(
                    "Stored size mismatch: header says {}, file has {}",
                    header.stored_size,
                    data.len()
                ),
            });
        }

        // === Integrity check: SHA-256(RAW DATA) == block_id ===
        // Decompress the raw data and verify identity
        let raw_data: Vec<u8> = if header.compression == Compression::None {
            data.clone()
        } else if header.compression == Compression::Zstd {
            let mut decoder =
                zstd::Decoder::new(&data[..]).map_err(|e| RepositoryError::General {
                    detail: format!("zstd decompression failed: {}", e),
                })?;
            let mut raw = Vec::with_capacity(header.raw_size as usize);
            decoder
                .read_to_end(&mut raw)
                .map_err(|e| RepositoryError::General {
                    detail: format!("Failed to read decompressed data: {}", e),
                })?;
            raw
        } else {
            return Err(RepositoryError::General {
                detail: format!("Unsupported compression: {:?}", header.compression),
            });
        };

        let expected_id = BlockId::from_raw_data(&raw_data);
        if expected_id != *id {
            return Err(RepositoryError::BlockCorrupted {
                expected: expected_id.to_hex(),
                actual: id.to_hex(),
            });
        }

        Ok(Block { header, data })
    }

    fn exists(&self, id: &BlockId) -> Result<bool, RepositoryError> {
        Ok(self.block_path(id).exists())
    }

    fn verify_block(&self, id: &BlockId) -> Result<bool, RepositoryError> {
        match self.get_block(id) {
            Ok(_) => Ok(true),
            Err(RepositoryError::BlockNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn root_path(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::block_store::block_header::Compression;
    use tempfile::TempDir;

    fn setup_store() -> (LocalFsBlockStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let store = LocalFsBlockStore::new(tmp.path().join("block-store"));
        (store, tmp)
    }

    #[test]
    fn test_put_and_get_block_none_compression() {
        let (store, _tmp) = setup_store();
        let raw_data = b"Hello, Nuwa Backup block store test!";
        let block_id = BlockId::from_raw_data(raw_data);

        let header = BlockHeader::new(
            Compression::None,
            raw_data.len() as u64,
            raw_data.len() as u64,
        );
        let block = Block {
            header,
            data: raw_data.to_vec(),
        };

        let stored_id = store.put_block(&block).unwrap();
        assert_eq!(stored_id, block_id);

        let retrieved = store.get_block(&block_id).unwrap();
        assert_eq!(retrieved.header.raw_size, raw_data.len() as u64);
        assert_eq!(retrieved.data, raw_data);
    }

    #[test]
    fn test_put_and_get_block_zstd_compression() {
        let (store, _tmp) = setup_store();
        let raw_data = b"This is some test data that will be compressed with zstd. ";

        let _header = BlockHeader::new(Compression::Zstd, raw_data.len() as u64, 0);
        // Compress the data
        let compressed = zstd::bulk::compress(&raw_data[..], 3).unwrap();
        let header = BlockHeader::new(
            Compression::Zstd,
            raw_data.len() as u64,
            compressed.len() as u64,
        );
        let block = Block {
            header,
            data: compressed,
        };

        let stored_id = store.put_block(&block).unwrap();
        let expected_id = BlockId::from_raw_data(raw_data);
        assert_eq!(stored_id, expected_id);

        let retrieved = store.get_block(&expected_id).unwrap();
        assert_eq!(retrieved.header.raw_size, raw_data.len() as u64);
    }

    #[test]
    fn test_block_exists() {
        let (store, _tmp) = setup_store();
        let raw_data = b"check if this block exists";
        let header = BlockHeader::new(
            Compression::None,
            raw_data.len() as u64,
            raw_data.len() as u64,
        );
        let block = Block {
            header,
            data: raw_data.to_vec(),
        };

        let id = store.put_block(&block).unwrap();
        assert!(store.exists(&id).unwrap());
    }

    #[test]
    fn test_block_not_found() {
        let (store, _tmp) = setup_store();
        let id = BlockId::from_raw_data(b"nonexistent");
        assert!(!store.exists(&id).unwrap());
        let result = store.get_block(&id);
        assert!(matches!(result, Err(RepositoryError::BlockNotFound(_))));
    }

    #[test]
    fn test_verify_integrity() {
        let (store, _tmp) = setup_store();
        let raw_data = b"verify my integrity please";
        let header = BlockHeader::new(
            Compression::None,
            raw_data.len() as u64,
            raw_data.len() as u64,
        );
        let block = Block {
            header,
            data: raw_data.to_vec(),
        };

        let id = store.put_block(&block).unwrap();
        assert!(store.verify_block(&id).unwrap());

        // Verify non-existent block returns false
        let fake_id = BlockId::from_raw_data(b"not stored");
        assert!(!store.verify_block(&fake_id).unwrap());
    }

    #[test]
    fn test_tampered_block_detected() {
        let (store, _tmp) = setup_store();
        let raw_data = b"do not tamper with my data";
        let header = BlockHeader::new(
            Compression::None,
            raw_data.len() as u64,
            raw_data.len() as u64,
        );
        let block = Block {
            header,
            data: raw_data.to_vec(),
        };

        let id = store.put_block(&block).unwrap();

        // Manually corrupt the stored block file
        let path = store.block_path(&id);
        let mut file_data = fs::read(&path).unwrap();
        // Corrupt a byte in the data section
        let corrupt_pos = BLOCK_HEADER_SIZE + 5;
        if corrupt_pos < file_data.len() {
            file_data[corrupt_pos] ^= 0xFF;
            fs::write(&path, &file_data).unwrap();
        }

        // Verification should now detect corruption
        let result = store.verify_block(&id);
        assert!(result.is_err() || !result.unwrap());
    }

    #[test]
    fn test_directory_structure() {
        let (store, _tmp) = setup_store();
        let raw_data = b"check directory layout";
        let header = BlockHeader::new(
            Compression::None,
            raw_data.len() as u64,
            raw_data.len() as u64,
        );
        let block = Block {
            header,
            data: raw_data.to_vec(),
        };

        let id = store.put_block(&block).unwrap();
        let path = store.block_path(&id);

        // Verify the directory layout: root/ab/12/hash.block
        let parent = path.parent().unwrap();
        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent, store.root.join(id.dir_prefix_1()));
        assert_eq!(
            parent,
            store.root.join(id.dir_prefix_1()).join(id.dir_prefix_2())
        );
    }
}
