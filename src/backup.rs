// ============================================================================
// backup.rs — 备份执行器（无 walkdir 依赖，手动递归遍历）
// ============================================================================

use crate::errors::NuwaError;
use crate::manifest::{CompressionConfig, DirectoryEntry, Manifest};
use crate::storage::BackupStorage;
use std::fs;
use std::path::{Path, PathBuf};

const SAFETY_MARGIN_RATIO: f64 = 0.10;

// 手动递归遍历目录（替代 walkdir）
// 设计原则：
// - 递归处理子目录
// - 跳过符号链接（原因：链接指向的文件可能不在备份范围内）
// - 返回扁平的文件列表和目录列表，供后续处理
fn walk_dir(dir: &Path, _base: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), NuwaError> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(dir.to_path_buf()),
        detail: format!("无法读取目录 '{}'", dir.display()),
        suggestion: "检查权限和路径".to_string(),
    })? {
        let entry = entry.map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dir.to_path_buf()),
            detail: "读取目录条目失败".to_string(),
            suggestion: "可能权限不足".to_string(),
        })?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(path.clone()),
            detail: "无法获取文件类型".to_string(),
            suggestion: "文件可能被删除".to_string(),
        })?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            dirs.push(path.clone());
            let (sf, sd) = walk_dir(&path, _base)?;
            files.extend(sf);
            dirs.extend(sd);
        } else if ft.is_file() {
            files.push(path);
        }
    }
    Ok((files, dirs))
}

fn to_relative(path: &Path, base: &Path) -> Result<String, NuwaError> {
    let r = path.strip_prefix(base).map_err(|_| NuwaError::General {
        detail: "路径前缀剥离失败".to_string(),
        suggestion: "内部错误".to_string(),
    })?;
    Ok(r.to_string_lossy().to_string().replace('\\', "/"))
}

fn calc_size(source: &Path) -> Result<u64, NuwaError> {
    let (files, _) = walk_dir(source, source)?;
    let mut total = 0u64;
    for f in &files {
        total += fs::metadata(f)?.len();
    }
    Ok(total)
}

/// 生成备份目录名：YYYYMMDD_HHMMSS_<源目录名>
// 格式说明：
// - 时间戳使用本地时间，方便用户按名称识别备份时间
// - 源目录名取自路径的最后一级
// - 如果发生冲突（同一秒两个备份），附加毫秒后缀
pub fn generate_backup_dir_name(source: &Path) -> String {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_string());
    format!("{}_{}", ts, name)
}

/// 执行完整备份
//
// 流程：
// 1. 验证源路径存在
// 2. 禁止源路径=目标路径（防止循环引用）
// 3. 检查目标路径可写
// 4. 计算总数据量（用于空间提示）
// 5. 创建备份存储结构
// 6. 遍历源目录，逐文件复制到备份存储
// 7. 写入 JSON Manifest
//
// 安全设计：
// - 原子写入：每个文件先写 .tmp → 完成后 rename
// - Manifest 也在完成后原子写入
// - 备份过程中断不会破坏已有备份
pub fn execute_backup(
    source: &Path,
    dest_root: &Path,
    compress: bool,
) -> Result<String, NuwaError> {
    // ===== 前置检查：源路径必须存在 =====
    // 必须在任何 canonicalize 或写入之前执行
    if !source.exists() {
        return Err(NuwaError::source_not_found(source));
    }

    // ===== 路径包含关系检查（零副作用）=====
    // 目标：在不创建目标目录的前提下判断包含关系。
    // 方案：对目标路径做 canonicalize_partial——找到最近存在的父目录 canonicalize，
    //       再拼接剩余路径组件。这样既解析了 8.3 短文件名，也统一了 \\?\ 前缀，
    //       且不产生任何文件系统副作用。
    fn canonicalize_partial(path: &Path) -> Result<PathBuf, NuwaError> {
        match std::fs::canonicalize(path) {
            Ok(p) => Ok(p),
            Err(_) => {
                let mut components: Vec<&std::ffi::OsStr> = Vec::new();
                let mut current = path;
                loop {
                    match current.file_name() {
                        Some(name) => {
                            components.push(name);
                            current = current.parent().unwrap_or_else(|| std::path::Path::new(""));
                            if current.exists() {
                                let base =
                                    std::fs::canonicalize(current).map_err(|e| NuwaError::Io {
                                        source: Some(e),
                                        path: Some(current.to_path_buf()),
                                        detail: format!("无法规范化路径 '{}'", current.display()),
                                        suggestion: "检查路径是否有效".to_string(),
                                    })?;
                                let result = components
                                    .into_iter()
                                    .rev()
                                    .fold(base, |acc, name| acc.join(name));
                                return Ok(result);
                            }
                        }
                        None => {
                            return Err(NuwaError::InvalidArgument {
                                detail: format!(
                                    "无法解析路径 '{}'：找不到任何存在的祖先目录",
                                    path.display()
                                ),
                                suggestion: "请确认路径格式正确".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    let src_canon = fs::canonicalize(source)?;
    let dst_canon = canonicalize_partial(dest_root)?;

    if src_canon == dst_canon {
        return Err(NuwaError::same_source_dest(source, dest_root));
    }

    // ===== 路径包含关系检查 =====
    // 使用 canonicalize 后的路径（前缀统一、短名已解析）做 Path::starts_with 比较
    if dst_canon.starts_with(&src_canon) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "目标路径位于源路径内部，禁止操作。
  源: {}
  目标: {}",
                src_canon.display(),
                dst_canon.display()
            ),
            suggestion: "请将备份目标选择在源目录之外。建议使用另一块硬盘或 USB 设备".to_string(),
        });
    }
    if src_canon.starts_with(&dst_canon) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "源路径位于目标路径内部，禁止操作。
  源: {}
  目标: {}",
                src_canon.display(),
                dst_canon.display()
            ),
            suggestion: "请将备份目标选择在源目录之外。建议使用另一块硬盘或 USB 设备".to_string(),
        });
    }

    // ===== 所有安全检查通过后，确保目标根目录存在 =====
    // 放在最后执行，避免因安全检查被拒绝而产生残留目录
    if !dest_root.exists() {
        std::fs::create_dir_all(dest_root).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_root.to_path_buf()),
            detail: format!("无法创建备份目标目录 '{}'", dest_root.display()),
            suggestion: "请检查目标路径是否有效且权限充足".to_string(),
        })?;
    } // ===== 计算备份数据总量，用于空间预估提示 =====
    let total = calc_size(source)?;
    // 保留 10% 安全余量
    let required = (total as f64 * (1.0 + SAFETY_MARGIN_RATIO)).ceil() as u64;

    // ===== 目标盘空间检查 =====
    // 使用 Win32 API GetDiskFreeSpaceExW 获取剩余空间
    // 在 Windows 上为精确检查，非 Windows 平台退化为可写性检查
    // 空间不足时直接返回错误，不产生半截备份
    crate::diskspace::check_disk_space(dest_root, required)?;

    let dir_name = generate_backup_dir_name(source);
    let store = BackupStorage::new(dest_root.to_path_buf(), dir_name.clone());
    // 如果目录名冲突，附加毫秒后缀
    let final_name = if store.backup_dir().exists() {
        format!("{}_{}", dir_name, chrono::Utc::now().format("%S%f"))
    } else {
        dir_name
    };
    let store = BackupStorage::new(dest_root.to_path_buf(), final_name.clone());
    store.create_backup_dirs()?;

    // ===== 检查压缩功能是否可用 =====
    // --compress 参数只在启用了 compress feature 时有效
    // 如果未启用但用户指定了 --compress，必须返回明确错误而非静默忽略
    #[cfg(not(feature = "compress"))]
    if compress {
        return Err(NuwaError::InvalidArgument {
            detail: "压缩功能未启用。当前程序未使用 compress feature 编译，--compress 参数无法工作".to_string(),
            suggestion: "请使用启用 compress 功能的版本（如 cargo build --features compress），或移除 --compress 参数".to_string(),
        });
    }

    // ===== 初始化 Manifest =====
    let mut manifest = Manifest::new(
        uuid::Uuid::new_v4().to_string(),
        src_canon.to_string_lossy().to_string(),
        CompressionConfig {
            enabled: compress,
            algorithm: if compress {
                Some("zstd".to_string())
            } else {
                None
            },
        },
    );

    // ===== 遍历源目录并复制所有文件 =====
    let (files, dirs) = walk_dir(source, source)?;
    for d in &dirs {
        manifest.add_directory(DirectoryEntry {
            relative_path: to_relative(d, source)?,
        });
    }
    for f in &files {
        let rel = to_relative(f, source)?;
        let fe = store.copy_file_with_atomic_write(f, &rel, compress)?;
        manifest.add_file(fe);
    }

    // ===== 原子写入 Manifest =====
    store.write_manifest(&manifest)?;
    Ok(final_name)
}
