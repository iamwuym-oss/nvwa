use crate::checksum;
use crate::errors::NuwaError;
use crate::manifest::{FileEntry, Manifest};
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct BackupStorage {
    pub dest_root: PathBuf,
    pub backup_dir_name: String,
}

impl BackupStorage {
    pub fn new(dest_root: PathBuf, backup_dir_name: String) -> Self {
        Self {
            dest_root,
            backup_dir_name,
        }
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.dest_root.join(&self.backup_dir_name)
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.backup_dir().join("manifest.json")
    }

    pub fn files_dir(&self) -> PathBuf {
        self.backup_dir().join("files")
    }

    pub fn stored_file_path(&self, relative_path: &str) -> PathBuf {
        let normalized = relative_path.replace('\\', "/");
        self.files_dir().join(&normalized)
    }

    pub fn create_backup_dirs(&self) -> Result<(), NuwaError> {
        std::fs::create_dir_all(self.backup_dir())?;
        std::fs::create_dir_all(self.files_dir())?;
        Ok(())
    }

    /// 带原子写入安全地将文件复制到备份存储
    /// 流程：先写 .tmp → 完成后 rename → 不存在"半截文件"状态
    pub fn copy_file_with_atomic_write(
        &self,
        src: &Path,
        relative_path: &str,
        enable_compression: bool,
    ) -> Result<FileEntry, NuwaError> {
        let sha256 = checksum::sha256_file(src)?;
        let metadata = std::fs::metadata(src)?;

        let modified_time = match metadata.modified() {
            Ok(mtime) => {
                let dt: chrono::DateTime<chrono::Utc> = mtime.into();
                dt.to_rfc3339()
            }
            Err(_) => "unknown".to_string(),
        };
        let size_bytes = metadata.len();
        let stored_path = relative_path.replace('\\', "/");
        let dest_path = self.stored_file_path(&stored_path);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let tmp_path = dest_path.with_extension("tmp");
        let mut src_file = std::fs::File::open(src)?;

        if enable_compression {
            #[cfg(feature = "compress")]
            {
                let tmp_file = std::fs::File::create(&tmp_path)?;
                let mut encoder = zstd::Encoder::new(tmp_file, 3)?;
                let mut buffer = vec![0u8; 64 * 1024];
                loop {
                    let bytes_read = src_file.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    encoder.write_all(&buffer[..bytes_read])?;
                }
                encoder.finish()?;
            }
            #[cfg(not(feature = "compress"))]
            {
                let _ = enable_compression;
                let mut dest_file = std::fs::File::create(&tmp_path)?;
                std::io::copy(&mut src_file, &mut dest_file)?;
            }
        } else {
            let mut dest_file = std::fs::File::create(&tmp_path)?;
            std::io::copy(&mut src_file, &mut dest_file)?;
        }

        // === 原子写入最后一步：重命名 .tmp → 最终文件 ===
        // 如果重命名失败，清理 .tmp 文件以防止残留
        match std::fs::rename(&tmp_path, &dest_path) {
            Ok(()) => {}
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(dest_path.clone()),
                    detail: "无法将临时文件移动到备份目录".to_string(),
                    suggestion: "检查目标磁盘空间和权限".to_string(),
                });
            }
        }

        Ok(FileEntry {
            relative_path: stored_path.clone(),
            size_bytes,
            modified_time,
            sha256,
            stored_path,
        })
    }

    /// 原子写入 manifest.json
    pub fn write_manifest(&self, manifest: &Manifest) -> Result<(), NuwaError> {
        let content = manifest.to_json_pretty()?;
        let manifest_path = self.manifest_path();
        let tmp_path = manifest_path.with_extension("json.tmp");

        std::fs::write(&tmp_path, content).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(tmp_path.clone()),
            detail: "无法写入 manifest 临时文件".to_string(),
            suggestion: "请检查目标磁盘空间是否充足".to_string(),
        })?;

        // === 原子写入：重命名 .json.tmp → manifest.json ===
        // 如果重命名失败，清理 .tmp 文件
        match std::fs::rename(&tmp_path, &manifest_path) {
            Ok(()) => {}
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(manifest_path.clone()),
                    detail: "无法重命名 manifest 临时文件".to_string(),
                    suggestion: "权限问题或磁盘空间不足".to_string(),
                });
            }
        }
        Ok(())
    }
}

pub fn read_manifest(backup_dir: &Path) -> Result<Manifest, NuwaError> {
    let manifest_path = backup_dir.join("manifest.json");
    Manifest::from_file(&manifest_path)
}

/// 恢复文件（原子写入）
pub fn restore_file(
    backup_dir: &Path,
    entry: &FileEntry,
    dest_path: &Path,
    was_compressed: bool,
) -> Result<(), NuwaError> {
    let src_path = backup_dir.join("files").join(&entry.stored_path);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = dest_path.with_extension("tmp");
    let mut src_file = std::fs::File::open(&src_path)?;

    if was_compressed {
        #[cfg(feature = "compress")]
        {
            let tmp_file = std::fs::File::create(&tmp_path)?;
            let mut decoder = zstd::Decoder::new(&mut src_file)?;
            let mut dest_file = tmp_file;
            std::io::copy(&mut decoder, &mut dest_file)?;
        }
        #[cfg(not(feature = "compress"))]
        {
            return Err(NuwaError::General {
                detail: "备份使用了压缩但当前程序未启用压缩支持".to_string(),
                suggestion: "请使用启用了 compress 功能的程序版本".to_string(),
            });
        }
    } else {
        let mut dest_file = std::fs::File::create(&tmp_path)?;
        std::io::copy(&mut src_file, &mut dest_file)?;
    }

    // === 原子写入：重命名 .tmp → 恢复目标 ===
    // 如果重命名失败，清理 .tmp 文件
    match std::fs::rename(&tmp_path, dest_path) {
        Ok(()) => {}
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(NuwaError::Io {
                source: Some(e),
                path: Some(dest_path.to_path_buf()),
                detail: "无法将恢复文件移动到目标位置".to_string(),
                suggestion: "检查目标路径的磁盘空间和权限".to_string(),
            });
        }
    }
    Ok(())
}
