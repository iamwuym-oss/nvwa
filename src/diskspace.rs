// ============================================================================
// diskspace.rs — 目标盘剩余空间检查
//
// 实现原理：
// 在 Windows 上通过 kernel32!GetDiskFreeSpaceExW API 获取目标路径所在磁盘的
// 剩余可用空间。这是 Windows 原生 API，无需额外 crate。
//
// 在非 Windows 平台上退化为可写性检查（与之前相同）。
//
// 安全设计：
// - 备份前检查剩余空间是否 >= 需要空间 × (1 + 安全余量)
// - 安全余量 = 10%，防止极端情况下的空间耗尽
// - 空间不足时返回带明确数字的错误，不产生半截备份
// ============================================================================

use crate::errors::NuwaError;
use std::path::Path;

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    // Win32 API: GetDiskFreeSpaceExW
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    /// 获取指定路径所在磁盘的剩余可用空间（字节）
    ///
    /// 返回 (可用空间, 总空间)
    pub fn free_space(path: &Path) -> Result<(u64, u64), NuwaError> {
        // 将路径转为 Windows 宽字符（UTF-16）格式
        let wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;

        // 安全调用 Win32 API
        // 注意：GetDiskFreeSpaceExW 返回 0 表示失败，非 0 表示成功
        let ret = unsafe {
            GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_bytes, &mut total_bytes, null_mut())
        };

        if ret == 0 {
            // API 调用失败（例如路径无效或无法访问）
            Err(NuwaError::Io {
                source: None,
                path: Some(path.to_path_buf()),
                detail: "无法查询目标磁盘空间（GetDiskFreeSpaceExW 调用失败）".to_string(),
                suggestion: "请确认目标路径是一个有效的磁盘路径".to_string(),
            })
        } else {
            Ok((free_bytes, total_bytes))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    /// 非 Windows 平台：空间检查不可用，退化为可写性检查
    pub fn free_space(path: &Path) -> Result<(u64, u64), NuwaError> {
        // 在非 Windows 平台上，无法可靠检查剩余空间
        // 返回 (0, 0) 表示不可用，调用方应退化为可写性检查
        Err(NuwaError::General {
            detail: format!("空间检查在非 Windows 平台上不可用：{}", path.display()),
            suggestion: "将执行基本的可写性检查".to_string(),
        })
    }
}

/// 检查目标路径是否有足够的空间容纳备份数据
///
/// ## 参数
/// * `dest_root` — 备份目标根路径
/// * `needed_bytes` — 备份数据所需的总字节数（含安全余量）
///
/// ## 返回值
/// * `Ok(())` — 空间充足
/// * `Err(NuwaError::Io { detail: "disk_space" })` — 空间不足
/// * `Err(...)` — 无法检查（退化为可写性检查）
///
/// ## 安全余量
/// 调用方应在 `needed_bytes` 中已包含 10% 安全余量
pub fn check_disk_space(dest_root: &Path, needed_bytes: u64) -> Result<(), NuwaError> {
    // Step 1: 确保目标路径存在（如果不存在则尝试创建）
    if !dest_root.exists() {
        std::fs::create_dir_all(dest_root).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_root.to_path_buf()),
            detail: "无法创建目标目录".to_string(),
            suggestion: "检查路径是否合法以及权限是否充足".to_string(),
        })?;
    }

    // Step 2: 尝试获取磁盘剩余空间
    match platform::free_space(dest_root) {
        Ok((free_bytes, _total_bytes)) => {
            // 检查可用空间是否足够
            if free_bytes < needed_bytes {
                return Err(NuwaError::disk_space(needed_bytes, free_bytes, dest_root));
            }

            // 空间充足，打印友好提示
            let free_mb = free_bytes / (1024 * 1024);
            let need_mb = needed_bytes / (1024 * 1024);
            if free_mb > 1024 {
                println!(
                    "✓ 目标盘空间充足：可用 {:.1} GB，需要 {} MB",
                    free_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                    need_mb
                );
            } else {
                println!("✓ 目标盘空间：可用 {} MB，需要 {} MB", free_mb, need_mb);
            }
            Ok(())
        }
        Err(_) => {
            // Step 3: API 不可用时，退化为可写性检查
            // 注意：这是 PARTIAL 实现，仅在无法调用 Win32 API 时降级
            let test_file = dest_root.join(".nuwa_space_check.tmp");
            match std::fs::write(&test_file, b"ok") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    println!("⚠ 无法检查磁盘剩余空间（仅验证了路径可写），请确保目标有足够空间");
                    println!("  预计需要约 {} MB 空间", needed_bytes / (1024 * 1024));
                    Ok(())
                }
                Err(e) => Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(dest_root.to_path_buf()),
                    detail: "目标路径不可写".to_string(),
                    suggestion: "检查权限和磁盘空间".to_string(),
                }),
            }
        }
    }
}
