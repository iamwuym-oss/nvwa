// ============================================================================
// errors.rs — Nüwa Backup 错误类型与用户友好的错误消息
//
// 设计原则：
// 1. 所有外部交互错误（IO、校验、参数）都有明确的错误类型
// 2. 不打印堆栈跟踪给普通用户（但保留 Debug 信息给开发者）
// 3. 每个错误都附带"用户可以怎么做"的建议
// 4. 退出码严格按照 AGENTS.md §13 的规范
// ============================================================================

use std::fmt;
use std::path::PathBuf;

/// Nüwa Backup 退出码 — 严格对照 Phase 1 CLI Contract
/// 参见 AGENTS.md §13 Exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,         // 操作成功完成
    GeneralFailure = 1,  // 通用错误，无法归入其他类别
    InvalidArgs = 2,     // 参数无效
    IoError = 3,         // I/O 错误（磁盘满、权限不足、文件锁定等）
    ChecksumFailure = 4, // 校验和不匹配或验证失败
    RestoreFailure = 5,  // 恢复验证失败
    SafetyViolation = 6, // 安全规则违反（如源路径=目标路径）
    ManifestError = 7,   // Manifest 缺失、损坏或不兼容
}

/// 统一的 Nüwa Backup 错误类型
/// 每个变体都封装了：
/// - 具体的技术错误（可选）
/// - 用户友好的中文错误描述
/// - 用户可以采取的操作建议
#[derive(Debug)]
pub enum NuwaError {
    /// 参数解析错误：用户输入了无效的命令行参数
    InvalidArgument { detail: String, suggestion: String },
    /// I/O 错误：文件读写、目录创建、磁盘空间等
    Io {
        source: Option<std::io::Error>,
        path: Option<PathBuf>,
        detail: String,
        suggestion: String,
    },
    /// 校验和错误：文件内容与记录不一致
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    /// 安全规则违反：操作可能损坏用户数据
    SafetyViolation { detail: String, suggestion: String },
    /// Manifest 错误：JSON 格式错误、版本不兼容、字段缺失
    ManifestError { detail: String, suggestion: String },
    /// 备份/恢复验证失败
    VerificationFailed { detail: String },
    /// 通用错误 — 兜底类型
    General { detail: String, suggestion: String },
}

impl fmt::Display for NuwaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NuwaError::InvalidArgument { detail, suggestion } => {
                write!(f, "参数错误：{}\n建议：{}", detail, suggestion)
            }
            NuwaError::Io {
                detail, suggestion, ..
            } => {
                write!(f, "I/O 错误：{}\n建议：{}", detail, suggestion)
            }
            NuwaError::ChecksumMismatch {
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "校验和不匹配：{} (预期: {}, 实际: {})",
                    path.display(),
                    expected,
                    actual
                )
            }
            NuwaError::SafetyViolation { detail, suggestion } => {
                write!(f, "安全规则禁止：{}\n建议：{}", detail, suggestion)
            }
            NuwaError::ManifestError { detail, suggestion } => {
                write!(f, "备份清单错误：{}\n建议：{}", detail, suggestion)
            }
            NuwaError::VerificationFailed { detail } => {
                write!(f, "验证失败：{}", detail)
            }
            NuwaError::General { detail, suggestion } => {
                write!(f, "错误：{}\n建议：{}", detail, suggestion)
            }
        }
    }
}

impl std::error::Error for NuwaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NuwaError::Io {
                source: Some(e), ..
            } => Some(e),
            _ => None,
        }
    }
}

/// 将 NuwaError 映射到对应的退出码
/// 主程序通过此方法决定进程退出时的返回值
impl From<&NuwaError> for ExitCode {
    fn from(err: &NuwaError) -> Self {
        match err {
            NuwaError::InvalidArgument { .. } => ExitCode::InvalidArgs,
            NuwaError::Io { .. } => ExitCode::IoError,
            NuwaError::ChecksumMismatch { .. } => ExitCode::ChecksumFailure,
            NuwaError::SafetyViolation { .. } => ExitCode::SafetyViolation,
            NuwaError::ManifestError { .. } => ExitCode::ManifestError,
            NuwaError::VerificationFailed { .. } => ExitCode::RestoreFailure,
            NuwaError::General { .. } => ExitCode::GeneralFailure,
        }
    }
}

/// 从 std::io::Error 方便地转换为 NuwaError
/// 自动提取路径信息（如果存在）并提供通用的 IO 错误建议
impl From<std::io::Error> for NuwaError {
    fn from(err: std::io::Error) -> Self {
        let detail = format!(
            "{} (错误码: {})",
            err,
            err.raw_os_error()
                .map_or_else(|| "未知".to_string(), |c| c.to_string())
        );
        NuwaError::Io {
            source: Some(err),
            path: None,
            detail,
            suggestion:
                "请检查：1) 目标路径是否存在且可写 2) 磁盘空间是否充足 3) 文件是否被其他程序占用"
                    .to_string(),
        }
    }
}

// ============================================================================
// 工厂方法 — 创建常见错误类型的便捷函数
// ============================================================================
impl NuwaError {
    /// 创建"源路径不存在"错误
    pub fn source_not_found(path: &std::path::Path) -> Self {
        NuwaError::InvalidArgument {
            detail: format!("源路径不存在：'{}'，请确认路径拼写正确", path.display()),
            suggestion: "使用绝对路径，例如：nuwa backup --source C:\\Users\\YourName\\Documents --dest D:\\Backup".to_string(),
        }
    }

    /// 创建"源路径 = 目标路径"安全错误
    pub fn same_source_dest(source: &std::path::Path, dest: &std::path::Path) -> Self {
        NuwaError::SafetyViolation {
            detail: format!(
                "源路径和目标路径相同，禁止操作。\n  源: {}\n  目标: {}",
                source.display(),
                dest.display()
            ),
            suggestion: "请选择不同的目标路径，建议使用另一块硬盘或 USB 设备".to_string(),
        }
    }

    /// 创建"磁盘空间不足"错误
    pub fn disk_space(needed: u64, available: u64, path: &std::path::Path) -> Self {
        let needed_mb = needed / (1024 * 1024);
        let available_mb = available / (1024 * 1024);
        NuwaError::Io {
            source: None,
            path: Some(path.to_path_buf()),
            detail: format!(
                "目标盘空间不足。需要约 {} MB，可用 {} MB。",
                needed_mb, available_mb
            ),
            suggestion:
                "请清理目标磁盘空间，或更换容量更大的备份目标。预留 10% 安全余量是强制要求。"
                    .to_string(),
        }
    }

    /// 创建 Manifest 解析错误
    pub fn manifest_parse(path: &std::path::Path, parse_err: &str) -> Self {
        NuwaError::ManifestError {
            detail: format!("无法解析备份清单文件：'{}'\n解析错误：{}", path.display(), parse_err),
            suggestion: "manifest.json 文件可能已损坏。请尝试：1) 运行 verify 命令检查备份完整性 2) 如果损坏，需重新创建备份".to_string(),
        }
    }

    /// 创建 Manifest 版本不兼容错误
    pub fn manifest_version(path: &std::path::Path, version: &str) -> Self {
        NuwaError::ManifestError {
            detail: format!(
                "备份清单版本不兼容：'{}' (版本: {})，当前程序仅支持版本 1.0",
                path.display(),
                version
            ),
            suggestion: "请使用创建此备份的相同版本程序进行恢复操作".to_string(),
        }
    }
}
