// ============================================================================
// manifest.rs — JSON Manifest 数据模型
// ============================================================================

use crate::errors::NuwaError;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub schema_version: String,
    pub backup_id: String,
    pub created_at: String,
    pub source_root: String,
    pub storage_format: String,
    pub compression: CompressionConfig,
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub summary: BackupSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub relative_path: String,
    pub size_bytes: u64,
    pub modified_time: String,
    pub sha256: String,
    pub stored_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryEntry {
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupSummary {
    pub file_count: u64,
    pub directory_count: u64,
    pub total_bytes: u64,
}

impl Manifest {
    pub fn new(backup_id: String, source_root: String, compression: CompressionConfig) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            backup_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            source_root,
            storage_format: "flat-file".to_string(),
            compression,
            files: Vec::new(),
            directories: Vec::new(),
            summary: BackupSummary {
                file_count: 0,
                directory_count: 0,
                total_bytes: 0,
            },
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, NuwaError> {
        serde_json::to_string_pretty(self).map_err(|e| NuwaError::General {
            detail: format!("JSON 序列化失败：{}", e),
            suggestion: "内部错误".to_string(),
        })
    }

    pub fn from_file(path: &Path) -> Result<Self, NuwaError> {
        let content = std::fs::read_to_string(path).map_err(|e| NuwaError::ManifestError {
            detail: format!("无法读取备份清单文件 '{}'：{}", path.display(), e),
            suggestion: "文件可能已损坏或权限不足".to_string(),
        })?;

        let manifest: Manifest = serde_json::from_str(&content)
            .map_err(|e| NuwaError::manifest_parse(path, &e.to_string()))?;

        manifest.validate_version(path)?;
        Ok(manifest)
    }

    fn validate_version(&self, path: &Path) -> Result<(), NuwaError> {
        let current_major = CURRENT_SCHEMA_VERSION.split('.').next().unwrap_or("0");
        let manifest_major = self.schema_version.split('.').next().unwrap_or("0");
        if current_major != manifest_major {
            return Err(NuwaError::manifest_version(path, &self.schema_version));
        }
        Ok(())
    }

    pub fn add_file(&mut self, entry: FileEntry) {
        self.summary.total_bytes += entry.size_bytes;
        self.summary.file_count += 1;
        self.files.push(entry);
    }

    pub fn add_directory(&mut self, entry: DirectoryEntry) {
        self.summary.directory_count += 1;
        self.directories.push(entry);
    }
}
