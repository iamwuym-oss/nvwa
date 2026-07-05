// ============================================================================
// main.rs — 程序入口
// ============================================================================

use nuwa_backup::cli::Command;
use nuwa_backup::errors::ExitCode;
use std::process;

fn main() {
    let cmd = match Command::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(ExitCode::InvalidArgs as i32);
        }
    };

    let exit = match cmd {
        Command::Backup {
            source,
            dest,
            compress,
        } => match nuwa_backup::backup::execute_backup(&source, &dest, compress) {
            Ok(dir) => {
                println!("\n✓ 备份完成！备份点目录: {}", dir);
                ExitCode::Success
            }
            Err(e) => {
                eprintln!("\n✗ 备份失败：{}", e);
                ExitCode::from(&e)
            }
        },
        Command::Restore {
            backup,
            dest,
            overwrite,
        } => match nuwa_backup::restore::execute_restore(&backup, &dest, overwrite) {
            Ok(r) => {
                if r.checksum_failures > 0 {
                    eprintln!("\n✗ 校验失败");
                    ExitCode::RestoreFailure
                } else {
                    println!("\n✓ 恢复完成！");
                    ExitCode::Success
                }
            }
            Err(e) => {
                eprintln!("\n✗ 恢复失败：{}", e);
                ExitCode::from(&e)
            }
        },
        Command::Verify { backup } => match nuwa_backup::verify::execute_verify(&backup) {
            Ok(_) => ExitCode::Success,
            Err(e) => {
                eprintln!("\n✗ 验证失败：{}", e);
                ExitCode::from(&e)
            }
        },
        Command::List { dest } => match nuwa_backup::list::execute_list(&dest) {
            Ok(s) => {
                nuwa_backup::list::print_list(&s);
                ExitCode::Success
            }
            Err(e) => {
                eprintln!("\n✗ 列出备份失败：{}", e);
                ExitCode::from(&e)
            }
        },
    };
    process::exit(exit as i32);
}
