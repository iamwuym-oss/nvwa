// ============================================================================
// backup_restore_tests.rs -- Phase 1 comprehensive tests
//
// All tests use std::env::temp_dir() + unique subdirectory,
// and clean up automatically after completion.
// Never writes to system directories, Desktop, Documents, etc.
// ============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ===== Test helper functions =====

fn test_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("nuwa_test_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("cannot create test dir");
    d
}

fn cleanup(d: &Path) {
    let _ = std::fs::remove_dir_all(d);
}

fn create_source(base: &Path) -> PathBuf {
    let src = base.join("source");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("hello.txt"), b"Hello Nuwa Backup").unwrap();
    std::fs::write(src.join("cn.txt"), b"Chinese test").unwrap();
    std::fs::write(src.join("jp_file.txt"), b"Japanese test").unwrap();
    std::fs::write(src.join("my file.txt"), b"spaces").unwrap();
    std::fs::create_dir_all(src.join("a").join("b").join("c")).unwrap();
    std::fs::create_dir_all(src.join("empty")).unwrap();
    std::fs::write(src.join("a").join("nested.txt"), b"nested").unwrap();
    src
}

fn create_many(base: &Path, n: usize) -> PathBuf {
    let d = base.join("many");
    std::fs::create_dir_all(&d).unwrap();
    for i in 0..n {
        std::fs::write(d.join(format!("f_{:04}.txt", i)), format!("data_{}\n", i)).unwrap();
    }
    d
}

fn checksums(dir: &Path) -> HashMap<String, String> {
    let mut m = HashMap::new();
    collect(dir, dir, &mut m);
    m
}

fn collect(dir: &Path, base: &Path, m: &mut HashMap<String, String>) {
    if !dir.is_dir() {
        return;
    }
    for e in std::fs::read_dir(dir).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            collect(&p, base, m);
        } else if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
            let rel = p
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .to_string()
                .replace("\\", "/");
            let h = nuwa_backup::checksum::sha256_file(&p).unwrap();
            m.insert(rel, h);
        }
    }
}

fn first_backup(dest: &Path) -> Option<PathBuf> {
    for e in std::fs::read_dir(dest).ok()? {
        let e = e.ok()?;
        let p = e.path();
        if p.is_dir() && p.join("manifest.json").exists() {
            return Some(p);
        }
    }
    None
}

/// 1. Normal file backup
#[test]
fn test_normal_backup() {
    let d = test_dir("backup");
    let src = create_source(&d);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    cleanup(&d);
}

/// 2. Normal file restore
#[test]
fn test_normal_restore() {
    let d = test_dir("restore");
    let src = create_source(&d);
    let dest = d.join("bk");
    let rd = d.join("rd");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    cleanup(&d);
}

/// 3. Backup -> delete source -> restore -> SHA-256 consistency
#[test]
fn test_consistency() {
    let d = test_dir("consist");
    let src = create_source(&d);
    let sh = checksums(&src);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    let rh = checksums(&rd);
    assert_eq!(sh.len(), rh.len(), "file count mismatch");
    for (k, v) in &sh {
        assert_eq!(rh.get(k), Some(v), "SHA-256 mismatch: {}", k);
    }
    cleanup(&d);
}

/// 4. Empty directory handling
#[test]
fn test_empty_dirs() {
    let d = test_dir("emptydir");
    let src = d.join("src");
    std::fs::create_dir_all(src.join("a").join("b")).unwrap();
    std::fs::create_dir_all(src.join("empty")).unwrap();
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    assert!(rd.join("empty").exists());
    assert!(rd.join("a").join("b").exists());
    cleanup(&d);
}

/// 5. Multi-level nested directories
#[test]
fn test_nested() {
    let d = test_dir("nested");
    let src = d.join("src");
    std::fs::create_dir_all(src.join("lvl1").join("lvl2").join("lvl3").join("lvl4")).unwrap();
    std::fs::write(src.join("lvl1").join("lvl2").join("deep.txt"), b"deep").unwrap();
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    assert!(rd.join("lvl1").join("lvl2").join("deep.txt").exists());
    assert!(rd
        .join("lvl1")
        .join("lvl2")
        .join("lvl3")
        .join("lvl4")
        .exists());
    cleanup(&d);
}

/// 6. Source path does not exist
#[test]
fn test_source_not_found() {
    let d = test_dir("notfound");
    let src = d.join("nonexistent");
    let dest = d.join("bk");
    let r = nuwa_backup::backup::execute_backup(&src, &dest, false);
    assert!(r.is_err());
    if let Err(ref e) = r {
        let code: nuwa_backup::errors::ExitCode = e.into();
        assert_eq!(
            code as i32,
            nuwa_backup::errors::ExitCode::InvalidArgs as i32
        );
    }
    cleanup(&d);
}

/// 7. Source path = destination path -> safety violation
#[test]
fn test_source_equals_dest() {
    let d = test_dir("samesrc");
    let src = d.join("mydata");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), b"data").unwrap();
    let r = nuwa_backup::backup::execute_backup(&src, &src, false);
    assert!(r.is_err());
    if let Err(ref e) = r {
        let code: nuwa_backup::errors::ExitCode = e.into();
        assert_eq!(
            code as i32,
            nuwa_backup::errors::ExitCode::SafetyViolation as i32
        );
    }
    cleanup(&d);
}

/// 8. Restore without overwrite when target file already exists
#[test]
fn test_no_overwrite() {
    let d = test_dir("nowrite");
    let src = d.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), b"original").unwrap();
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    std::fs::create_dir_all(&rd).unwrap();
    std::fs::write(rd.join("file.txt"), b"user data").unwrap();
    let r = nuwa_backup::restore::execute_restore(&bp, &rd, false);
    assert!(r.is_ok(), "restore without overwrite should skip, not fail");
    let result = r.unwrap();
    assert_eq!(result.restored_count, 0);
    assert_eq!(result.skipped_count, 1);
    let content = std::fs::read_to_string(rd.join("file.txt")).unwrap();
    assert_eq!(content, "user data");
    cleanup(&d);
}

/// 9. Corrupted manifest -> verify must fail
#[test]
fn test_corrupted_manifest() {
    let d = test_dir("corruptman");
    let src = create_source(&d);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    std::fs::write(bp.join("manifest.json"), b"not valid json{").unwrap();
    assert!(nuwa_backup::verify::execute_verify(&bp).is_err());
    cleanup(&d);
}

/// 10. Backed-up data file tampered -> verify must fail
#[test]
fn test_tampered_data() {
    let d = test_dir("tamper");
    let src = create_source(&d);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let f = bp.join("files").join("hello.txt");
    if f.exists() {
        std::fs::write(&f, b"tampered content").unwrap();
    }
    assert!(nuwa_backup::verify::execute_verify(&bp).is_err());
    cleanup(&d);
}

/// 11. List command shows correct number of backup points
#[test]
fn test_list() {
    let d = test_dir("list");
    let src = create_source(&d);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let s = nuwa_backup::list::execute_list(&dest).expect("list should succeed");
    assert_eq!(s.len(), 2, "should have 2 backup points");
    cleanup(&d);
}

/// 12. Bulk 100 small files
#[test]
fn test_100_files() {
    let d = test_dir("100files");
    let src = create_many(&d, 100);
    let sh = checksums(&src);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    let rh = checksums(&rd);
    assert_eq!(sh.len(), rh.len());
    for (k, v) in &sh {
        assert_eq!(rh.get(k), Some(v), "SHA-256: {}", k);
    }
    cleanup(&d);
}

/// 13. Destination path not writable -> backup must fail
#[test]
fn test_dest_not_writable() {
    let d = test_dir("nowrite2");
    let src = create_source(&d);
    let dest_file = d.join("is_a_file.txt");
    std::fs::write(&dest_file, b"blocker").unwrap();
    let r = nuwa_backup::backup::execute_backup(&src, &dest_file, false);
    assert!(r.is_err());
    cleanup(&d);
}

/// 14. Bulk 1000 small files
#[test]
fn test_1000_files() {
    let d = test_dir("1000files");
    let src = create_many(&d, 1000);
    let sh = checksums(&src);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let rd = d.join("rd");
    assert!(nuwa_backup::restore::execute_restore(&bp, &rd, true).is_ok());
    let rh = checksums(&rd);
    assert_eq!(sh.len(), rh.len());
    for (k, v) in &sh {
        assert_eq!(rh.get(k), Some(v), "SHA-256: {}", k);
    }
    cleanup(&d);
}

/// 15. Backup interrupt safety test
#[test]
fn test_interrupt_safety() {
    let d = test_dir("interrupt");
    let src = create_source(&d);
    let dest = d.join("bk");
    assert!(nuwa_backup::backup::execute_backup(&src, &dest, false).is_ok());
    let bp = first_backup(&dest).expect("backup point exists");
    let files_dir = bp.join("files");
    if files_dir.exists() {
        std::fs::write(files_dir.join("hello.txt.tmp"), b"crash residue").ok();
        std::fs::write(files_dir.join("orphan.tmp"), b"orphan").ok();
        let _ = std::fs::create_dir_all(files_dir.join("a").join("b"));
        std::fs::write(
            files_dir.join("a").join("b").join("nested.txt.tmp"),
            b"deep crash",
        )
        .ok();
    }
    assert!(nuwa_backup::verify::execute_verify(&bp).is_ok());
    cleanup(&d);
}

/// 16. Destination path inside source path -> forbidden
#[test]
fn test_dest_inside_source() {
    let d = test_dir("destinsrc");
    let src = d.join("source");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), b"data").unwrap();
    let dest = src.join("backup");
    let r = nuwa_backup::backup::execute_backup(&src, &dest, false);
    assert!(r.is_err());
    if let Err(ref e) = r {
        let code: nuwa_backup::errors::ExitCode = e.into();
        assert_eq!(
            code as i32,
            nuwa_backup::errors::ExitCode::SafetyViolation as i32
        );
    }
    cleanup(&d);
}

/// 17. Source path inside destination path -> forbidden
#[test]
fn test_source_inside_dest() {
    let d = test_dir("srcindest");
    let root = d.join("root");
    let src = root.join("source");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), b"data").unwrap();
    let dest = d.join("root");
    let r = nuwa_backup::backup::execute_backup(&src, &dest, false);
    assert!(r.is_err());
    if let Err(ref e) = r {
        let code: nuwa_backup::errors::ExitCode = e.into();
        assert_eq!(
            code as i32,
            nuwa_backup::errors::ExitCode::SafetyViolation as i32
        );
    }
    cleanup(&d);
}

/// 18. Lock check must NOT modify existing file content
///
/// Verify that is_file_locked() does not change file content, size, or checksum.
/// This is a safety-critical test: if lock detection truncates or modifies the file,
/// user data would be destroyed before restore even begins.
#[test]
fn test_lock_check_does_not_modify_existing_file() {
    let d = test_dir("locknomod");
    let f = d.join("target.txt");
    std::fs::write(
        &f,
        b"important user data that must never be modified by lock check",
    )
    .unwrap();
    let orig_meta = std::fs::metadata(&f).unwrap();
    let orig_size = orig_meta.len();
    let orig_sha = nuwa_backup::checksum::sha256_file(&f).unwrap();

    // Call is_file_locked (the function used during restore to check lock status)
    let result = nuwa_backup::restore::is_file_locked(&f);
    assert!(
        result.is_ok(),
        "is_file_locked should not fail on an unlocked file"
    );
    assert!(!result.unwrap(), "file should not be reported as locked");

    // Verify file content is unchanged
    let after_meta = std::fs::metadata(&f).unwrap();
    let after_size = after_meta.len();
    let after_sha = nuwa_backup::checksum::sha256_file(&f).unwrap();

    assert_eq!(
        orig_size, after_size,
        "file size must not change after lock check"
    );
    assert_eq!(
        orig_sha, after_sha,
        "file SHA-256 must not change after lock check"
    );
    cleanup(&d);
}

/// 19. Path containment check must NOT create dest directory when rejected
///
/// When dest is inside source, the operation is rejected with SafetyViolation.
/// The dest directory must NOT be created as a side effect.
#[test]
fn test_dest_inside_source_does_not_create_dest_dir() {
    let d = test_dir("nocreatedest");
    let src = d.join("source");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), b"data").unwrap();
    let dest = src.join("backup_subdir");

    // dest does not exist yet
    assert!(!dest.exists(), "dest must not exist before backup attempt");

    let r = nuwa_backup::backup::execute_backup(&src, &dest, false);
    assert!(r.is_err(), "dest inside source must be rejected");

    // dest must NOT have been created by the failed operation
    assert!(
        !dest.exists(),
        "dest must not be created when operation is rejected"
    );

    if let Err(ref e) = r {
        let code: nuwa_backup::errors::ExitCode = e.into();
        assert_eq!(
            code as i32,
            nuwa_backup::errors::ExitCode::SafetyViolation as i32,
            "must return SafetyViolation (exit code 6)"
        );
    }
    cleanup(&d);
}
