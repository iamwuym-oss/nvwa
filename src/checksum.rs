// ============================================================================
// checksum.rs -- SHA-256 checksum utilities
// ============================================================================

use sha2::{Digest, Sha256};
use std::io::{self, Read};
use std::path::Path;

const READ_BUF_SIZE: usize = 64 * 1024;

pub fn sha256_file(path: &Path) -> Result<String, crate::errors::NuwaError> {
    let file = std::fs::File::open(path)?;
    let mut reader = io::BufReader::with_capacity(READ_BUF_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; READ_BUF_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn verify_file_checksum(path: &Path, expected: &str) -> Result<bool, crate::errors::NuwaError> {
    let actual = sha256_file(path)?;
    Ok(actual == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty file SHA-256 should match the known standard value
    #[test]
    fn test_sha256_empty_file() {
        let dir = std::env::temp_dir().join("nuwa_test_sha256_e");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("empty.txt");
        std::fs::write(&path, b"").unwrap();
        let hash = sha256_file(&path).unwrap();
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// SHA-256 format verification
    #[test]
    fn test_sha256_format() {
        let dir = std::env::temp_dir().join("nuwa_test_sha256_fmt");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("known.txt");
        std::fs::write(&path, b"Hello, Nuwa Backup!").unwrap();
        let hash = sha256_file(&path).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Memory and file SHA-256 should match
    #[test]
    fn test_sha256_bytes_consistency() {
        let data = b"Consistency test data";
        let bytes_hash = sha256_bytes(data);
        let dir = std::env::temp_dir().join("nuwa_test_sha256_cons");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("cons.txt");
        std::fs::write(&path, data).unwrap();
        let file_hash = sha256_file(&path).unwrap();
        assert_eq!(bytes_hash, file_hash);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
