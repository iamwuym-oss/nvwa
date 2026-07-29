use std::fs;

use fixture_generator::{generate, verify, DEFAULT_SEED, MANIFEST_NAME};
use tempfile::tempdir;

// TST-FIX-001: identical seed and dataset version produce identical manifests.
#[test]
fn deterministic_generation_produces_identical_manifests() {
    let first = tempdir().expect("temporary directory");
    let second = tempdir().expect("temporary directory");
    let first_manifest = generate(first.path(), DEFAULT_SEED).expect("first generation");
    let second_manifest = generate(second.path(), DEFAULT_SEED).expect("second generation");

    assert_eq!(first_manifest, second_manifest);
    assert_eq!(
        fs::read(first.path().join(MANIFEST_NAME)).expect("first manifest"),
        fs::read(second.path().join(MANIFEST_NAME)).expect("second manifest")
    );
}

// TST-FIX-002: approved boundaries, patterns, deep paths and Unicode are self-verified.
#[test]
fn generated_dataset_covers_approved_cases_and_self_verifies() {
    let fixture = tempdir().expect("temporary directory");
    let manifest = generate(fixture.path(), DEFAULT_SEED).expect("generation");
    let paths: Vec<_> = manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();

    assert!(paths.contains(&"boundaries/empty.bin"));
    assert!(paths.contains(&"boundaries/one-byte.bin"));
    assert!(paths.contains(&"boundaries/4k.bin"));
    assert!(paths.contains(&"boundaries/256k.bin"));
    assert!(paths.contains(&"patterns/all-zero-64k.bin"));
    assert!(paths.contains(&"patterns/repeated-64k.bin"));
    assert!(paths.contains(&"random/seeded-64k.bin"));
    assert!(paths.contains(&"deep/a/b/c/d/e/payload.txt"));
    assert!(paths.iter().any(|path| path.starts_with("unicode/")));
    assert_eq!(verify(fixture.path()).expect("verification"), manifest);
}

// TST-FIX-003: file content tampering is detected.
#[test]
fn verification_rejects_content_tampering() {
    let fixture = tempdir().expect("temporary directory");
    generate(fixture.path(), DEFAULT_SEED).expect("generation");
    fs::write(fixture.path().join("boundaries/one-byte.bin"), b"x").expect("tamper file");

    assert!(verify(fixture.path()).is_err());
}

// TST-FIX-004: extra files and non-empty generation targets are rejected.
#[test]
fn generation_and_verification_reject_unexpected_files() {
    let non_empty = tempdir().expect("temporary directory");
    fs::write(non_empty.path().join("existing.txt"), b"keep").expect("existing file");
    assert!(generate(non_empty.path(), DEFAULT_SEED).is_err());

    let fixture = tempdir().expect("temporary directory");
    generate(fixture.path(), DEFAULT_SEED).expect("generation");
    fs::write(fixture.path().join("unexpected.txt"), b"unexpected").expect("extra file");
    assert!(verify(fixture.path()).is_err());
}

// TST-FIX-005: manifest path traversal and manifest/data co-tampering are rejected.
#[test]
fn verification_rejects_unsafe_or_unapproved_manifests() {
    let fixture = tempdir().expect("temporary directory");
    generate(fixture.path(), DEFAULT_SEED).expect("generation");
    let manifest_path = fixture.path().join(MANIFEST_NAME);
    let original = fs::read_to_string(&manifest_path).expect("manifest");
    let unsafe_manifest = original.replacen(
        "\"path\": \"boundaries/256k.bin\"",
        "\"path\": \"../escaped.bin\"",
        1,
    );
    fs::write(&manifest_path, unsafe_manifest).expect("unsafe manifest");
    assert!(verify(fixture.path()).is_err());

    generate_fresh_and_tamper_both();
}

fn generate_fresh_and_tamper_both() {
    let fixture = tempdir().expect("temporary directory");
    let mut manifest = generate(fixture.path(), DEFAULT_SEED).expect("generation");
    let target = fixture.path().join("boundaries/one-byte.bin");
    fs::write(&target, b"z").expect("tamper file");
    let entry = manifest
        .files
        .iter_mut()
        .find(|entry| entry.path == "boundaries/one-byte.bin")
        .expect("entry");
    entry.sha256 = "594e519ae499312b29433b7dd8a97ff068defcba9755b6d5d00e84c524d67b06".to_owned();
    fs::write(
        fixture.path().join(MANIFEST_NAME),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("serialize")
        ),
    )
    .expect("tamper manifest");
    assert!(verify(fixture.path()).is_err());
}
