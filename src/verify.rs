// ============================================================================
// verify.rs -- Backup verification
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
        "Verifying backup '{}' (created at {})",
        manifest.backup_id, manifest.created_at
    );
    println!("Total files: {}", result.total_files);

    for entry in &manifest.files {
        let stored_path = backup_dir.join("files").join(&entry.stored_path);

        if !stored_path.exists() {
            println!("File missing: {}", entry.relative_path);
            result.inaccessible += 1;
            result.failed_files.push(entry.relative_path.clone());
            continue;
        }

        match checksum::verify_file_checksum(&stored_path, &entry.sha256) {
            Ok(true) => {
                result.passed += 1;
            }
            Ok(false) => {
                println!("Checksum mismatch: {}", entry.relative_path);
                result.failed += 1;
                result.failed_files.push(entry.relative_path.clone());
            }
            Err(e) => {
                println!("Cannot verify: {} ({})", entry.relative_path, e);
                result.inaccessible += 1;
                result.failed_files.push(entry.relative_path.clone());
            }
        }
    }

    println!("\nPassed: {}/{}", result.passed, result.total_files);
    if result.failed > 0 {
        println!("Checksum failures: {}", result.failed);
    }
    if result.inaccessible > 0 {
        println!("Inaccessible: {}", result.inaccessible);
    }

    if result.failed > 0 || result.inaccessible > 0 {
        println!("\nRecommendation: delete this backup point and recreate the backup");
        return Err(NuwaError::VerificationFailed {
            detail: format!(
                "{} files have checksum mismatches, {} files are inaccessible",
                result.failed, result.inaccessible
            ),
        });
    }

    println!("\nBackup verification passed");
    Ok(result)
}
