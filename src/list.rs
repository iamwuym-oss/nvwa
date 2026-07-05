// ============================================================================
// list.rs — Nüwa Backup 备份点列表
//
// 设计原则：
// 1. 扫描目标路径下的所有备份点目录
// 2. 读取每个备份点的 manifest.json 获取元信息
// 3. 按时间倒序排列（最新的排最前）
// 4. 显示每个备份点的：时间、源路径、文件数、总大小
// 5. 对于 manifest 损坏的备份点，依然列出并标记为"异常"
//    — 原因：损坏的备份点仍可让用户知道"曾经有这个备份"
//    — 但会明确提示用户需要运行 verify 检查
// ============================================================================

use crate::errors::NuwaError;
use crate::manifest::Manifest;
use std::path::Path;

/// 单个备份点的摘要信息
pub struct BackupPointSummary {
    /// 备份目录名称
    pub dir_name: String,
    /// 备份目录的完整路径
    pub full_path: String,
    /// 备份 ID（UUID）
    pub backup_id: String,
    /// 备份创建时间
    pub created_at: String,
    /// 源路径
    pub source_root: String,
    /// 文件数量
    pub file_count: u64,
    /// 总大小（字节）
    pub total_bytes: u64,
    /// Manifest 是否可正常读取
    pub manifest_ok: bool,
}

/// 执行列表操作
///
/// ## 参数
/// * `dest_path` — 备份目标根路径
///
/// ## 流程
/// 1. 遍历目标根路径下的所有子目录
/// 2. 尝试读取每个子目录的 manifest.json
/// 3. 汇总所有备份点信息
///
/// ## 返回
/// * `Vec<BackupPointSummary>` — 所有备份点的信息列表
pub fn execute_list(dest_path: &Path) -> Result<Vec<BackupPointSummary>, NuwaError> {
    // ===== 检查目标路径是否存在 =====
    if !dest_path.exists() {
        return Err(NuwaError::InvalidArgument {
            detail: format!("目标路径不存在：'{}'", dest_path.display()),
            suggestion:
                "请确认备份目标路径是否正确。使用绝对路径，例如：nuwa list --dest D:\\Backup"
                    .to_string(),
        });
    }

    if !dest_path.is_dir() {
        return Err(NuwaError::InvalidArgument {
            detail: format!("目标路径不是目录：'{}'", dest_path.display()),
            suggestion: "请输入备份根目录的路径，而非文件路径".to_string(),
        });
    }

    let mut summaries: Vec<BackupPointSummary> = Vec::new();

    // ===== 遍历目标根路径下的所有条目 =====
    let entries = std::fs::read_dir(dest_path).map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(dest_path.to_path_buf()),
        detail: "无法读取目标目录".to_string(),
        suggestion: "请检查目录权限和路径是否正确".to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_path.to_path_buf()),
            detail: "遍历目录时出错".to_string(),
            suggestion: "可能某个子目录权限不足".to_string(),
        })?;

        let path = entry.path();

        // 只处理子目录（每个备份点是一个目录）
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // 跳过隐藏目录（以 . 开头）
        // 原因：避免将系统目录或临时目录误认为备份点
        if dir_name.starts_with('.') {
            continue;
        }

        // ===== 尝试读取 Manifest =====
        let manifest_path = path.join("manifest.json");
        let summary = if manifest_path.exists() {
            match Manifest::from_file(&manifest_path) {
                Ok(manifest) => BackupPointSummary {
                    dir_name: dir_name.clone(),
                    full_path: path.to_string_lossy().to_string(),
                    backup_id: manifest.backup_id,
                    created_at: manifest.created_at,
                    source_root: manifest.source_root,
                    file_count: manifest.summary.file_count,
                    total_bytes: manifest.summary.total_bytes,
                    manifest_ok: true,
                },
                Err(_) => {
                    // Manifest 损坏 — 仍列出但标记异常
                    BackupPointSummary {
                        dir_name: dir_name.clone(),
                        full_path: path.to_string_lossy().to_string(),
                        backup_id: "未知".to_string(),
                        created_at: "未知".to_string(),
                        source_root: "未知 (Manifest 损坏)".to_string(),
                        file_count: 0,
                        total_bytes: 0,
                        manifest_ok: false,
                    }
                }
            }
        } else {
            // 没有 manifest.json — 跳过非备份目录
            continue;
        };

        summaries.push(summary);
    }

    // ===== 按时间倒序排列（最新的排最前） =====
    // 原因：用户通常先查看最近的备份点
    // 如果创建时间未知，排在最后
    summaries.sort_by(|a, b| {
        if a.manifest_ok && b.manifest_ok {
            b.created_at.cmp(&a.created_at)
        } else if b.manifest_ok {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });

    Ok(summaries)
}

/// 打印备份点列表到控制台
pub fn print_list(summaries: &[BackupPointSummary]) {
    if summaries.is_empty() {
        println!("没有找到备份点。");
        println!("请先执行 nuwa backup --source <path> --dest <path> 创建备份。");
        return;
    }

    println!("备份目标中的备份点 (共 {} 个):\n", summaries.len());

    for (i, s) in summaries.iter().enumerate() {
        if !s.manifest_ok {
            println!("  {}. ⚠ [异常] {}", i + 1, s.dir_name);
            println!("     路径: {}", s.full_path);
            println!("     Manifest 损坏，请运行 verify 检查\n");
            continue;
        }

        // 格式化文件大小
        let size_str = if s.total_bytes > 1024 * 1024 * 1024 {
            format!(
                "{:.2} GB",
                s.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        } else if s.total_bytes > 1024 * 1024 {
            format!("{:.2} MB", s.total_bytes as f64 / (1024.0 * 1024.0))
        } else if s.total_bytes > 1024 {
            format!("{:.2} KB", s.total_bytes as f64 / 1024.0)
        } else {
            format!("{} B", s.total_bytes)
        };

        println!("  {}. {}", i + 1, s.dir_name);
        println!("     备份时间: {}", s.created_at);
        println!("     源路径:   {}", s.source_root);
        println!("     文件数:   {} | 总大小: {}", s.file_count, size_str);
        println!("     备份 ID:  {}", s.backup_id);
        println!();
    }
}
