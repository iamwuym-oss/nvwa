// ============================================================================
// verify.rs — 备份验证器
// ============================================================================

use crate::checksum;
use crate::errors::NuwaError;
use crate::storage::read_manifest;
use std::path::Path;

pub struct VerifyResult {
    pub total_files: u64,
    pub passed: u64,
    pub failed: u64,
    pub inaccessible: u64,
    pub failed_files: Vec<String>,
}

pub fn execute_verify(backup_dir: &Path) -> Result<VerifyResult, NuwaError> {
    let manifest = read_manifest(backup_dir)?;

    let mut result = VerifyResult {
        total_files: manifest.files.len() as u64,
        passed: 0,
        failed: 0,
        inaccessible: 0,
        failed_files: Vec::new(),
    };

    println!(
        "验证备份 '{}' (创建于 {})",
        manifest.backup_id, manifest.created_at
    );
    println!("共 {} 个文件", result.total_files);

    for entry in &manifest.files {
        let stored_path = backup_dir.join("files").join(&entry.stored_path);

        if !stored_path.exists() {
            println!("✗ 文件缺失：{}", entry.relative_path);
            result.inaccessible += 1;
            result.failed_files.push(entry.relative_path.clone());
            continue;
        }

        match checksum::verify_file_checksum(&stored_path, &entry.sha256) {
            Ok(true) => {
                result.passed += 1;
            }
            Ok(false) => {
                println!("✗ 校验和不匹配：{}", entry.relative_path);
                result.failed += 1;
                result.failed_files.push(entry.relative_path.clone());
            }
            Err(e) => {
                println!("✗ 无法验证：{} ({})", entry.relative_path, e);
                result.inaccessible += 1;
                result.failed_files.push(entry.relative_path.clone());
            }
        }
    }

    println!("\n通过: {}/{}", result.passed, result.total_files);
    if result.failed > 0 {
        println!("校验失败: {}", result.failed);
    }
    if result.inaccessible > 0 {
        println!("无法访问: {}", result.inaccessible);
    }

    if result.failed > 0 || result.inaccessible > 0 {
        println!("\n建议：删除此备份点并重新创建备份");
        return Err(NuwaError::VerificationFailed {
            detail: format!(
                "{} 个文件校验不匹配，{} 个文件无法访问",
                result.failed, result.inaccessible
            ),
        });
    }

    println!("\n✓ 备份验证通过");
    Ok(result)
}
