// ============================================================================
// block_header.rs 鈥?64-byte Block Header encoding/decoding
// ============================================================================
//
// Block Header is a fixed 64-byte binary structure embedded at the start
// of every block file in block-store/. See Architecture v1.0 搂5.3.
//
// Key design rules:
// - Reserved fields are set to zero and MUST be ignored on read
// - CRC32C covers only the header itself (offset 0-59), not the data
// - Data integrity is provided by filename = SHA-256(raw) (block_id)

use crate::repository::error::RepositoryError;
use std::path::PathBuf;

pub const BLOCK_HEADER_SIZE: usize = 64;

/// Compression algorithms supported in Phase S
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None = 0,
    Zstd = 1,
}

impl Compression {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            0 => Some(Compression::None),
            1 => Some(Compression::Zstd),
            _ => None,
        }
    }

    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

/// Fixed 64-byte header at the start of each block file.
///
/// Layout:
/// | Offset | Size | Field          | Description                  |
/// |--------|------|----------------|------------------------------|
/// | 0      | 4    | magic          | b"NWBL"                      |
/// | 4      | 2    | version        | 1                            |
/// | 6      | 10   | reserved       | Zero (future flags)          |
/// | 16     | 2    | hash_algorithm  | 0 = SHA-256                  |
/// | 18     | 2    | compression    | 0 = none, 1 = zstd           |
/// | 20     | 8    | raw_size       | Uncompressed data size       |
/// | 28     | 8    | stored_size    | Stored (compressed) size     |
/// | 36     | 24   | reserved       | Zero (future extension)      |
/// | 60     | 4    | header_crc     | CRC32C of bytes 0-59         |
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub version: u16,
    pub hash_algorithm: u16,
    pub compression: Compression,
    pub raw_size: u64,
    pub stored_size: u64,
}

impl BlockHeader {
    pub fn new(compression: Compression, raw_size: u64, stored_size: u64) -> Self {
        BlockHeader {
            version: 1,
            hash_algorithm: 0, // SHA-256
            compression,
            raw_size,
            stored_size,
        }
    }

    /// Encode header into a 64-byte array
    pub fn encode(&self) -> [u8; BLOCK_HEADER_SIZE] {
        let mut buf = [0u8; BLOCK_HEADER_SIZE];

        // Magic: b"NWBL" at offset 0
        buf[0..4].copy_from_slice(b"NWBL");

        // Version: u16 at offset 4 (little-endian)
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());

        // Reserved: bytes 6-15 remain zero

        // Hash algorithm: u16 at offset 16
        buf[16..18].copy_from_slice(&self.hash_algorithm.to_le_bytes());

        // Compression: u16 at offset 18
        buf[18..20].copy_from_slice(&self.compression.to_u16().to_le_bytes());

        // Raw size: u64 at offset 20
        buf[20..28].copy_from_slice(&self.raw_size.to_le_bytes());

        // Stored size: u64 at offset 28
        buf[28..36].copy_from_slice(&self.stored_size.to_le_bytes());

        // Reserved: bytes 36-59 remain zero

        // CRC32C of bytes 0-59 at offset 60
        let crc = crc32c(&buf[0..60]);
        buf[60..64].copy_from_slice(&crc.to_le_bytes());

        buf
    }

    /// Decode header from a 64-byte buffer
    pub fn decode(buf: &[u8]) -> Result<Self, RepositoryError> {
        let path = PathBuf::from("<block header decode>");

        if buf.len() < BLOCK_HEADER_SIZE {
            return Err(RepositoryError::invalid_block_header(
                path,
                format!("Expected {} bytes, got {}", BLOCK_HEADER_SIZE, buf.len()),
            ));
        }

        // Validate magic
        if &buf[0..4] != b"NWBL" {
            return Err(RepositoryError::invalid_block_header(
                path,
                format!("Invalid magic: expected NWBL, got {:?}", &buf[0..4]),
            ));
        }

        // Validate CRC32C
        let stored_crc = u32::from_le_bytes(
            buf[60..64]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );
        let computed_crc = crc32c(&buf[0..60]);
        if stored_crc != computed_crc {
            return Err(RepositoryError::invalid_block_header(
                path,
                format!(
                    "CRC mismatch: stored {:#x}, computed {:#x}",
                    stored_crc, computed_crc
                ),
            ));
        }

        let version = u16::from_le_bytes(
            buf[4..6]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );
        let hash_algo = u16::from_le_bytes(
            buf[16..18]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );
        let compression_val = u16::from_le_bytes(
            buf[18..20]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );
        let raw_size = u64::from_le_bytes(
            buf[20..28]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );
        let stored_size = u64::from_le_bytes(
            buf[28..36]
                .try_into()
                .expect("validated BLOCK_HEADER_SIZE buffer"),
        );

        let compression = Compression::from_u16(compression_val).ok_or_else(|| {
            RepositoryError::invalid_block_header(
                path,
                format!("Unsupported compression type: {}", compression_val),
            )
        })?;

        Ok(BlockHeader {
            version,
            hash_algorithm: hash_algo,
            compression,
            raw_size,
            stored_size,
        })
    }
}

/// Simple CRC32C (Castagnoli) implementation using the polynomial 0x1EDC6F41
fn crc32c(data: &[u8]) -> u32 {
    let table: [u32; 256] = {
        let poly: u32 = 0x1EDC6F41;
        let mut table = [0u32; 256];
        for (i, item) in table.iter_mut().enumerate() {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ poly;
                } else {
                    crc >>= 1;
                }
            }
            *item = crc;
        }
        table
    };

    let mut crc = !0u32;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ table[idx];
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode_roundtrip() {
        let header = BlockHeader::new(Compression::Zstd, 262144, 131072);
        let encoded = header.encode();
        let decoded = BlockHeader::decode(&encoded).unwrap();

        assert_eq!(decoded.version, header.version);
        assert_eq!(decoded.hash_algorithm, header.hash_algorithm);
        assert_eq!(decoded.compression, header.compression);
        assert_eq!(decoded.raw_size, header.raw_size);
        assert_eq!(decoded.stored_size, header.stored_size);
    }

    #[test]
    fn test_header_none_compression() {
        let header = BlockHeader::new(Compression::None, 4096, 4096);
        let encoded = header.encode();
        let decoded = BlockHeader::decode(&encoded).unwrap();
        assert_eq!(decoded.compression, Compression::None);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut buf = [0u8; BLOCK_HEADER_SIZE];
        buf[0..4].copy_from_slice(b"BAD!");
        assert!(BlockHeader::decode(&buf).is_err());
    }

    #[test]
    fn test_header_tampered_crc() {
        let header = BlockHeader::new(Compression::Zstd, 100, 50);
        let mut encoded = header.encode();
        encoded[0] ^= 0xFF; // Tamper byte 0
        assert!(BlockHeader::decode(&encoded).is_err());
    }

    #[test]
    fn test_header_size() {
        assert_eq!(BLOCK_HEADER_SIZE, 64);
    }

    #[test]
    fn test_header_reserved_fields_zero() {
        let header = BlockHeader::new(Compression::Zstd, 12345, 6789);
        let encoded = header.encode();
        // Bytes 6-15: reserved
        for (i, _) in encoded.iter().enumerate().take(16).skip(6) {
            assert_eq!(encoded[i], 0, "Reserved byte {} is not zero", i);
        }
        // Bytes 36-59: reserved
        for (i, _) in encoded.iter().enumerate().take(60).skip(36) {
            assert_eq!(encoded[i], 0, "Reserved byte {} is not zero", i);
        }
    }
}
