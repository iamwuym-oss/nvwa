// ============================================================================
// restore.rs — Nüwa Backup 恢复执行器
//
// 核心流程：
// 1. 读取备份目录的 manifest.json
// 2. 重新创建所有目录结构
// 3. 逐文件恢复：
//    a. 如果目标文件已存在且 --overwrite=false → 跳过
//    b. 如果目标文件已存在且 --overwrite=true → 检查是否被锁定
//    c. 如果被锁定 → 跳过并报告
//    d. 否则 → 原子写入目标文件
//    e. 恢复后验证 SHA-256
// 4. 输出恢复总结
// ============================================================================

use crate::checksum;
use crate::errors::NuwaError;
use crate::storage::{self, read_manifest};
use std::fs::OpenOptions;
use std::path::Path;

/// 恢复操作的结果统计
pub struct RestoreResult {
    /// 成功恢复的文件数
    pub restored_count: u64,
    /// 跳过的文件数（已存在且 --overwrite=false，或文件被锁定）
    pub skipped_count: u64,
    /// 校验和失败的文件数
    pub checksum_failures: u64,
}

/// 执行文件级恢复
//
// 参数：
// - backup_dir: 备份点目录路径（包含 manifest.json 和 files/ 子目录）
// - dest: 恢复目标路径
// - overwrite: 是否覆盖已存在的文件
//
// 安全设计：
// - 不会覆盖已存在的文件（除非指定 --overwrite）
// - 会检测文件是否被其他进程锁定（通过尝试写入打开）
// - 使用原子写入（.tmp → rename）保证恢复过程中断不产生损坏文件
// - 恢复后逐个验证文件 SHA-256
pub fn execute_restore(
    backup_dir: &Path,
    dest: &Path,
    overwrite: bool,
) -> Result<RestoreResult, NuwaError> {
    let manifest = read_manifest(backup_dir)?;
    let compression = manifest.compression.enabled;

    // ===== 第一步：重建所有目录 =====
    // 先创建目录再恢复文件，确保目录结构完整
    for dir_entry in &manifest.directories {
        std::fs::create_dir_all(dest.join(&dir_entry.relative_path))?;
    }

    let mut result = RestoreResult {
        restored_count: 0,
        skipped_count: 0,
        checksum_failures: 0,
    };

    // ===== 第二步：逐文件恢复 =====
    for file_entry in &manifest.files {
        let dest_path = dest.join(&file_entry.relative_path);

        // 情况1：文件已存在且未指定 --overwrite → 跳过
        if dest_path.exists() && !overwrite {
            println!("跳过 (文件已存在)：{}", file_entry.relative_path);
            result.skipped_count += 1;
            continue;
        }

        // 情况2：文件已存在且指定了 --overwrite
        // 但文件可能被其他进程锁定，需要先检查
        if dest_path.exists() && overwrite {
            match is_file_locked(&dest_path) {
                Ok(true) => {
                    // 文件被锁定，跳过并提示用户
                    println!("跳过 (文件被其他进程占用)：{}", file_entry.relative_path);
                    println!("  提示：请关闭占用此文件的程序后重试");
                    result.skipped_count += 1;
                    continue;
                }
                Ok(false) => {
                    // 文件未被锁定，可以恢复
                }
                Err(e) => {
                    // 检测失败（如路径问题），返回错误
                    return Err(e);
                }
            }
        }

        // ===== 执行恢复（原子写入） =====
        storage::restore_file(backup_dir, file_entry, &dest_path, compression)?;

        // ===== 恢复后验证 SHA-256 =====
        match checksum::verify_file_checksum(&dest_path, &file_entry.sha256) {
            Ok(true) => {
                result.restored_count += 1;
                println!("恢复成功：{}", file_entry.relative_path);
            }
            Ok(false) => {
                result.checksum_failures += 1;
                println!("⚠ 校验和不匹配：{}", file_entry.relative_path);
            }
            Err(e) => {
                return Err(NuwaError::VerificationFailed {
                    detail: format!("恢复后验证失败 ({}): {}", file_entry.relative_path, e),
                });
            }
        }
    }

    // ===== 第三步：输出总结 =====
    println!(
        "\n恢复完成：{} 成功，{} 跳过，{} 校验失败",
        result.restored_count, result.skipped_count, result.checksum_failures
    );

    if result.checksum_failures > 0 {
        return Err(NuwaError::VerificationFailed {
            detail: format!("{} 个文件恢复后校验和不匹配", result.checksum_failures),
        });
    }

    Ok(result)
}

/// 检查目标文件是否被其他进程锁定
//
// 实现原理：
// 尝试以写入模式打开已有文件（不创建、不截断）。
// 在 Windows 上，如果一个没有 FILE_SHARE_WRITE 共享标志的进程正在使用此文件，
// CreateFileW 会返回 "Permission Denied"。
// 在 Linux/macOS 上，写入锁定的文件会返回 "Resource temporarily unavailable"。
//
// 返回值：
// - Ok(true)  → 文件被锁定，无法写入
// - Ok(false) → 文件未被锁定，可以安全写入
// - Err(...)  → 检测过程出错
pub fn is_file_locked(path: &Path) -> Result<bool, NuwaError> {
    if !path.exists() {
        // 文件不存在肯定没有被锁定
        return Ok(false);
    }

    // 以写入模式打开但不创建新文件、不截断现有内容
    // 这只是锁检测，不应修改文件
    match OpenOptions::new()
        .write(true) // 请求写入权限
        .create(false) // 不创建新文件
        .truncate(false) // 不截断现有文件
        .open(path)
    {
        Ok(_file) => {
            // 打开成功 → 文件未被锁定
            // _file 在此作用域结束时自动关闭
            Ok(false)
        }
        Err(e) => {
            // 根据错误类型判断
            match e.kind() {
                // Windows: 独占锁会返回 PermissionDenied
                // POSIX: 写入锁会返回 WouldBlock
                std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::WouldBlock => Ok(true),
                _ => Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(path.to_path_buf()),
                    detail: format!("无法检测文件状态：{}", path.display()),
                    suggestion: "检查文件是否被其他程序占用".to_string(),
                }),
            }
        }
    }
}
