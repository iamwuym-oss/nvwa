// ============================================================================
// cli.rs — CLI 参数解析（手动解析，无 clap 依赖）
// ============================================================================

use crate::errors::NuwaError;
use std::path::PathBuf;

pub enum Command {
    Backup {
        source: PathBuf,
        dest: PathBuf,
        compress: bool,
    },
    Restore {
        backup: PathBuf,
        dest: PathBuf,
        overwrite: bool,
    },
    Verify {
        backup: PathBuf,
    },
    List {
        dest: PathBuf,
    },
}

impl Command {
    pub fn from_args() -> Result<Self, NuwaError> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            return Err(NuwaError::InvalidArgument {
                detail: "缺少子命令".to_string(),
                suggestion: Self::usage(),
            });
        }

        match args[1].to_lowercase().as_str() {
            "backup" => Self::parse_backup(&args[2..]),
            "restore" => Self::parse_restore(&args[2..]),
            "verify" => Self::parse_verify(&args[2..]),
            "list" => Self::parse_list(&args[2..]),
            "--help" | "-h" => {
                println!("{}", Self::usage_full());
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("nuwa-backup v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => Err(NuwaError::InvalidArgument {
                detail: format!("未知子命令 '{}'", args[1]),
                suggestion: Self::usage(),
            }),
        }
    }

    fn parse_backup(args: &[String]) -> Result<Self, NuwaError> {
        let mut source = None;
        let mut dest = None;
        let mut compress = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--source" => {
                    i += 1;
                    source = Some(self::next_arg(args, i, "--source")?);
                }
                "--dest" => {
                    i += 1;
                    dest = Some(self::next_arg(args, i, "--dest")?);
                }
                "--compress" => {
                    compress = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("未知参数 '{}'", args[i]),
                        suggestion: "用法: nuwa backup --source <路径> --dest <路径> [--compress]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Backup {
            source: source.ok_or_else(|| missing_param("--source"))?,
            dest: dest.ok_or_else(|| missing_param("--dest"))?,
            compress,
        })
    }

    fn parse_restore(args: &[String]) -> Result<Self, NuwaError> {
        let mut backup = None;
        let mut dest = None;
        let mut overwrite = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--backup" => {
                    i += 1;
                    backup = Some(self::next_arg(args, i, "--backup")?);
                }
                "--dest" => {
                    i += 1;
                    dest = Some(self::next_arg(args, i, "--dest")?);
                }
                "--overwrite" => {
                    overwrite = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("未知参数 '{}'", args[i]),
                        suggestion:
                            "用法: nuwa restore --backup <备份目录> --dest <目标路径> [--overwrite]"
                                .to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Restore {
            backup: backup.ok_or_else(|| missing_param("--backup"))?,
            dest: dest.ok_or_else(|| missing_param("--dest"))?,
            overwrite,
        })
    }

    fn parse_verify(args: &[String]) -> Result<Self, NuwaError> {
        let mut backup = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--backup" => {
                    i += 1;
                    backup = Some(self::next_arg(args, i, "--backup")?);
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("未知参数 '{}'", args[i]),
                        suggestion: "用法: nuwa verify --backup <备份目录>".to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Verify {
            backup: backup.ok_or_else(|| missing_param("--backup"))?,
        })
    }

    fn parse_list(args: &[String]) -> Result<Self, NuwaError> {
        let mut dest = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--dest" => {
                    i += 1;
                    dest = Some(self::next_arg(args, i, "--dest")?);
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("未知参数 '{}'", args[i]),
                        suggestion: "用法: nuwa list --dest <备份根目录>".to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::List {
            dest: dest.ok_or_else(|| missing_param("--dest"))?,
        })
    }

    fn usage() -> String {
        "用法: nuwa <子命令> [参数]\n  子命令: backup, restore, verify, list\n  nuwa --help 查看详细帮助".to_string()
    }

    fn usage_full() -> String {
        let v = env!("CARGO_PKG_VERSION");
        format!(
            r#"Nüwa Backup v{}
文件级备份与恢复工具

子命令:
  backup    完整备份
  restore   恢复
  verify    验证
  list      列出备份点

backup 参数:
  --source <路径>   源路径（必填）
  --dest <路径>     目标路径（必填）
  --compress        启用压缩（可选）

restore 参数:
  --backup <路径>   备份目录（必填）
  --dest <路径>     恢复目标（必填）
  --overwrite       覆盖已存在文件（可选）

verify 参数:
  --backup <路径>   备份目录（必填）

list 参数:
  --dest <路径>     备份根目录（必填）
"#,
            v
        )
    }
}

fn next_arg(args: &[String], idx: usize, param: &str) -> Result<PathBuf, NuwaError> {
    if idx >= args.len() {
        Err(NuwaError::InvalidArgument {
            detail: format!("参数 {} 缺少值", param),
            suggestion: format!("{} 后需要跟路径参数", param),
        })
    } else {
        Ok(PathBuf::from(&args[idx]))
    }
}

fn missing_param(param: &str) -> NuwaError {
    NuwaError::InvalidArgument {
        detail: format!("缺少 {} 参数", param),
        suggestion: format!("请使用 --{} <路径> 指定", param),
    }
}
