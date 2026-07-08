// ============================================================================
// file_browser_service_tests.rs -- Test the in-app file browser service
//
// Test scenarios:
//   1. list_roots returns usable result on current platform
//   2. list_directory returns child directories
//   3. list_directory sorts entries by name
//   4. list_directory handles missing path with clear error
//   5. list_directory handles file path input with clear error
//   6. Permission denied should not panic (simulated)
// ============================================================================

use nuwa_backup::app::services::file_browser_service;

// ---------------------------------------------------------------------------
// list_roots
// ---------------------------------------------------------------------------

#[test]
fn test_list_roots_returns_non_empty() {
    let roots = file_browser_service::list_roots().expect("list_roots should succeed");
    // On Windows, at minimum C:\ should exist
    // On other platforms, / should exist
    assert!(!roots.is_empty(), "should return at least one root entry");
    for entry in &roots {
        assert!(!entry.path.is_empty(), "root path must not be empty");
        assert!(!entry.label.is_empty(), "root label must not be empty");
    }
}

#[test]
fn test_list_roots_contains_system_drive() {
    let roots = file_browser_service::list_roots().expect("list_roots should succeed");

    #[cfg(target_os = "windows")]
    {
        let has_c = roots.iter().any(|r| r.path == "C:\\");
        assert!(has_c, "C:\\ should be listed on Windows");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let has_root = roots.iter().any(|r| r.path == "/");
        assert!(has_root, "/ should be listed on Unix");
    }
}

// ---------------------------------------------------------------------------
// list_directory
// ---------------------------------------------------------------------------

#[test]
fn test_list_directory_returns_child_dirs() {
    let dir = std::env::temp_dir();
    let entries = file_browser_service::list_directory(&dir.to_string_lossy())
        .expect("list_directory on temp dir should succeed");

    // Temp dir should have at least some entries
    // We can't guarantee content, but the call should succeed
    assert!(
        entries.iter().all(|e| e.is_directory),
        "all entries must be directories"
    );
}

#[test]
fn test_list_directory_sorted_by_name() {
    use std::fs;
    let tmp = std::env::temp_dir().join(format!("nuwa_test_sort_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create test dir");

    // Create test subdirs with out-of-order names
    for name in &["zeta", "alpha", "beta", "Gamma"] {
        fs::create_dir_all(tmp.join(name)).expect("create subdir");
    }

    let entries = file_browser_service::list_directory(&tmp.to_string_lossy())
        .expect("list_directory should succeed");

    // Collect names in order
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

    // Verify sorted (case-insensitive)
    let mut sorted = names.clone();
    sorted.sort_by_key(|n| n.to_lowercase());
    assert_eq!(
        names, sorted,
        "entries must be sorted case-insensitively by name"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_list_directory_missing_path() {
    let result = file_browser_service::list_directory("C:\\__nuwa_test_nonexistent_path_12345__");

    assert!(result.is_err(), "missing path should return error");
    let err = result.unwrap_err();
    assert_eq!(
        err.category, "Internal",
        "missing path should be Internal category"
    );
}

#[test]
fn test_list_directory_file_path() {
    // Use a known system file
    let test_path = if cfg!(target_os = "windows") {
        "C:\\Windows\\System32\\notepad.exe"
    } else {
        "/bin/sh"
    };

    let result = file_browser_service::list_directory(test_path);
    assert!(result.is_err(), "file path should return error");
}

#[test]
fn test_list_directory_empty_dir() {
    use std::fs;
    let tmp = std::env::temp_dir().join(format!("nuwa_test_empty_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create empty test dir");

    let entries = file_browser_service::list_directory(&tmp.to_string_lossy())
        .expect("list_directory should succeed on empty dir");

    assert!(
        entries.is_empty(),
        "empty directory should return empty list"
    );

    let _ = fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_list_directory_long_path() {
    // A very long but nonexistent path
    let long = format!("C:\\{}", "a\\".repeat(50));
    let result = file_browser_service::list_directory(&long);
    assert!(result.is_err(), "very long non-existent path should error");
}

#[test]
fn test_list_directory_root() {
    #[cfg(target_os = "windows")]
    {
        let entries =
            file_browser_service::list_directory("C:\\").expect("listing root C:\\ should succeed");
        assert!(
            !entries.is_empty(),
            "C:\\ root should have directory entries"
        );
    }
}
