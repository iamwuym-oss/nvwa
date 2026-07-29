use std::process::Command;

// TST-VSN-005: real product CLI --version shows Draft identity.
#[test]
fn cli_version_shows_draft_identity() {
    let output = Command::new(env!("CARGO_BIN_EXE_nuwa-backup"))
        .arg("--version")
        .output()
        .expect("nuwa-backup --version must run");
    assert!(output.status.success(), "exit code must be 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must contain product version line
    assert!(stdout.contains("nuwa-backup"), "must show product name");

    // Must contain NWB format line
    assert!(stdout.contains("NWB format"), "must show NWB format");

    // Must contain Draft restriction
    assert!(stdout.contains("DRAFT"), "must contain DRAFT");
    assert!(
        stdout.contains("internal testing only"),
        "must contain internal testing only"
    );
    assert!(
        stdout.contains("not for release"),
        "must contain not for release"
    );
}
