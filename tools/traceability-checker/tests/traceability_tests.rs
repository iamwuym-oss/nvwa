use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;
use traceability_checker::{
    check_workspace, load_registry, render_matrix, validate_registry, CheckError, Registry,
    REGISTRY_PATH,
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("traceability-checker must be located below the workspace root")
        .to_path_buf()
}

fn committed_registry() -> Registry {
    load_registry(&workspace_root()).expect("committed traceability registry must parse")
}

fn assert_semantic_contains(
    result: Result<traceability_checker::CheckReport, CheckError>,
    needle: &str,
) {
    match result {
        Err(CheckError::Semantic(message)) => assert!(
            message.contains(needle),
            "expected semantic error containing {needle:?}, got {message:?}"
        ),
        other => panic!("expected semantic error containing {needle:?}, got {other:?}"),
    }
}

fn copy_fixture() -> TempDir {
    let source_root = workspace_root();
    let registry = committed_registry();
    let fixture = tempfile::tempdir().expect("temporary fixture directory must be created");
    let mut paths: BTreeSet<_> = registry.source_allowlist.iter().cloned().collect();
    paths.insert(REGISTRY_PATH.to_owned());
    for locator in &registry.locators {
        paths.insert(locator.path.clone());
        if let Some(fixture) = &locator.fixture {
            paths.insert(fixture.clone());
        }
        if let Some(stderr) = &locator.stderr {
            paths.insert(stderr.clone());
        }
    }
    for relative in paths {
        let destination = fixture.path().join(&relative);
        fs::create_dir_all(
            destination
                .parent()
                .expect("fixture file must have a parent"),
        )
        .expect("fixture parent directory must be created");
        fs::copy(source_root.join(&relative), &destination)
            .expect("allowlisted fixture source must be copied");
    }
    let matrix = fixture.path().join(&registry.matrix_path);
    fs::create_dir_all(matrix.parent().expect("matrix must have a parent"))
        .expect("matrix fixture directory must be created");
    fs::write(matrix, "stale matrix\n").expect("stale matrix fixture must be written");
    fixture
}

// TST-TRC-001: a valid registry passes all semantic and source checks.
#[test]
fn valid_registry_passes() {
    let report = validate_registry(&committed_registry(), &workspace_root())
        .expect("registry must validate");
    assert_eq!(report.requirements, 38);
    assert_eq!(report.tests, 150);
    assert_eq!(report.mappings, 153);
    assert_eq!(report.p0_requirements, 35);
    assert_eq!(report.planned_tests, 115);
    assert_eq!(report.implemented_tests, 35);
    assert_eq!(report.mapped_tests, 131);
    assert_eq!(report.source_scoped_tests, 19);
    let registry = committed_registry();
    assert!(registry
        .tests
        .iter()
        .filter(|test| test.status == "PLANNED")
        .all(|test| registry
            .locators
            .iter()
            .all(|locator| locator.test != test.id)));
}

// TST-TRC-002: Markdown rendering is byte-deterministic.
#[test]
fn markdown_generation_is_deterministic() {
    let registry = committed_registry();
    let report = validate_registry(&registry, &workspace_root()).expect("registry must validate");
    let first = render_matrix(&registry, &report);
    let second = render_matrix(&registry, &report);
    assert_eq!(first, second);

    let fixture = copy_fixture();
    let matrix_path = fixture.path().join(&registry.matrix_path);
    let windows = format!("\u{feff}{}", first.replace('\n', "\r\n"));
    fs::write(matrix_path, windows).expect("BOM/CRLF matrix fixture must be written");
    check_workspace(fixture.path()).expect("BOM/CRLF matrix must compare deterministically");
}

// TST-TRC-003: duplicate requirement IDs are rejected.
#[test]
fn duplicate_requirement_is_rejected() {
    let mut registry = committed_registry();
    registry.requirements.push(registry.requirements[0].clone());
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "duplicate requirement ID",
    );
}

// TST-TRC-004: duplicate test IDs are rejected.
#[test]
fn duplicate_test_is_rejected() {
    let mut registry = committed_registry();
    registry.tests.push(registry.tests[0].clone());
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "duplicate test ID",
    );
}

// TST-TRC-005: duplicate requirement/test mapping pairs are rejected.
#[test]
fn duplicate_mapping_is_rejected() {
    let mut registry = committed_registry();
    registry.mappings.push(registry.mappings[0].clone());
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "duplicate mapping",
    );

    let mut registry = committed_registry();
    registry.mappings[0].role = "OBSERVATION".to_owned();
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "illegal role",
    );

    let mut registry = committed_registry();
    registry.mappings[0].rationale.clear();
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "empty rationale",
    );
}

// TST-TRC-006: illegal formal IDs are rejected.
#[test]
fn illegal_id_is_rejected() {
    let mut registry = committed_registry();
    registry.requirements[0].id = "REQ-1".to_owned();
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "illegal requirement ID",
    );
    assert!(traceability_checker::valid_test_id("TST-A-1B-001"));
    assert!(traceability_checker::valid_test_id("TST-A-123-001"));
    assert!(!traceability_checker::valid_test_id("TST-1A-ABC-001"));
    assert!(!traceability_checker::valid_test_id("TST-A--001"));
    assert!(!traceability_checker::valid_test_id("TST-A-b-001"));
}

// TST-TRC-007: a mapping cannot reference an unknown requirement.
#[test]
fn unknown_requirement_reference_is_rejected() {
    let mut registry = committed_registry();
    registry.mappings[0].requirement = "REQ-999".to_owned();
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "unknown requirement",
    );
}

// TST-TRC-008: a mapping cannot reference an unknown test.
#[test]
fn unknown_test_reference_is_rejected() {
    let mut registry = committed_registry();
    registry.mappings[0].test = ["TST", "UNKNOWN", "999"].join("-");
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "unknown test",
    );
}

// TST-TRC-009: every P0 requirement needs a positive test.
#[test]
fn p0_without_positive_test_is_rejected() {
    let mut registry = committed_registry();
    for mapping in &mut registry.mappings {
        if mapping.requirement == "REQ-002" && mapping.role == "POSITIVE" {
            mapping.role = "NEGATIVE".to_owned();
        }
    }
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "lacks a POSITIVE test",
    );
}

// TST-TRC-010: every P0 requirement needs a negative, fault, or security test.
#[test]
fn p0_without_negative_test_is_rejected() {
    let mut registry = committed_registry();
    for mapping in &mut registry.mappings {
        if mapping.requirement == "REQ-002" && mapping.role != "POSITIVE" {
            mapping.role = "POSITIVE".to_owned();
        }
    }
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "lacks a NEGATIVE, FAULT, or SECURITY test",
    );
}

// TST-TRC-011: the registry cannot silently define an empty P0 denominator.
#[test]
fn zero_p0_requirements_is_rejected() {
    let mut registry = committed_registry();
    for requirement in &mut registry.requirements {
        requirement.priority = "P1".to_owned();
    }
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "zero P0 requirements",
    );
}

// TST-TRC-012: mapped and source-scoped dispositions enforce their edge contracts.
#[test]
fn orphan_test_is_rejected() {
    let mut registry = committed_registry();
    registry
        .mappings
        .retain(|mapping| mapping.test != "TST-FILE-001");
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "MAPPED test TST-FILE-001 has no requirement edge",
    );

    validate_registry(&committed_registry(), &workspace_root())
        .expect("a legitimate SOURCE_SCOPED test with no edge must pass");

    let mut registry = committed_registry();
    let mut forbidden_edge = registry.mappings[0].clone();
    forbidden_edge.requirement = "REQ-008".to_owned();
    forbidden_edge.test = "TST-CRY-001".to_owned();
    forbidden_edge.role = "POSITIVE".to_owned();
    forbidden_edge.rationale = "forbidden source-scoped edge".to_owned();
    registry.mappings.push(forbidden_edge);
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "SOURCE_SCOPED test TST-CRY-001 must have zero requirement edges",
    );

    let mut registry = committed_registry();
    let scoped = registry
        .tests
        .iter_mut()
        .find(|test| test.id == "TST-CRY-001")
        .expect("source-scoped test exists");
    scoped.scope_source = None;
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "missing authority scope metadata",
    );

    let mut registry = committed_registry();
    let scoped = registry
        .tests
        .iter_mut()
        .find(|test| test.id == "TST-CRY-001")
        .expect("source-scoped test exists");
    scoped.scope_justification = Some(String::new());
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "empty scope justification",
    );
}

// TST-TRC-013: missing or ambiguous source evidence is rejected.
#[test]
fn invalid_source_evidence_is_rejected() {
    let mut registry = committed_registry();
    registry.requirements[0].excerpt = "an excerpt that does not exist".to_owned();
    assert_semantic_contains(
        validate_registry(&registry, &workspace_root()),
        "source anchor/excerpt must each match exactly once",
    );

    let fixture = copy_fixture();
    let mut registry = load_registry(fixture.path()).expect("fixture registry must parse");
    let source = fixture.path().join(&registry.requirements[0].source);
    let duplicate = registry.requirements[0].anchor.clone();
    let mut contents = fs::read_to_string(&source).expect("source fixture must be readable");
    contents.push_str(&format!("\n{duplicate}\n"));
    fs::write(source, contents).expect("ambiguous source fixture must be writable");
    assert_semantic_contains(
        validate_registry(&registry, fixture.path()),
        "source anchor/excerpt must each match exactly once",
    );

    registry.requirements[0].source = registry.source_allowlist[1].clone();
    assert_semantic_contains(
        validate_registry(&registry, fixture.path()),
        "registry P source ID inventory",
    );

    let extra_native = copy_fixture();
    let registry = load_registry(extra_native.path()).expect("fixture registry must parse");
    let format_source = registry
        .requirements
        .iter()
        .find(|requirement| requirement.id == "FMT-001")
        .expect("format requirement must exist")
        .source
        .clone();
    let format_path = extra_native.path().join(format_source);
    let mut contents = fs::read_to_string(&format_path).expect("format fixture must be readable");
    contents.push_str("\nInjected native requirement FMT-011.\n");
    fs::write(format_path, contents).expect("format fixture must be writable");
    assert_semantic_contains(
        validate_registry(&registry, extra_native.path()),
        "authoritative FMT ID inventory",
    );

    let replaced_native = copy_fixture();
    let registry = load_registry(replaced_native.path()).expect("fixture registry must parse");
    let format_source = registry
        .requirements
        .iter()
        .find(|requirement| requirement.id == "FMT-001")
        .expect("format requirement must exist")
        .source
        .clone();
    let format_path = replaced_native.path().join(format_source);
    let contents = fs::read_to_string(&format_path)
        .expect("format fixture must be readable")
        .replace("FMT-001", "FMT-099");
    fs::write(format_path, contents).expect("format fixture must be writable");
    assert_semantic_contains(
        validate_registry(&registry, replaced_native.path()),
        "authoritative FMT ID inventory",
    );

    let substring_collision = copy_fixture();
    let registry = load_registry(substring_collision.path()).expect("fixture registry must parse");
    let format_source = registry
        .requirements
        .iter()
        .find(|requirement| requirement.id == "FMT-001")
        .expect("format requirement must exist")
        .source
        .clone();
    let format_path = substring_collision.path().join(format_source);
    let mut contents = fs::read_to_string(&format_path).expect("format fixture must be readable");
    contents.push_str("\nSubstring collision FMT-011X is not a native requirement ID.\n");
    fs::write(format_path, contents).expect("format fixture must be writable");
    validate_registry(&registry, substring_collision.path())
        .expect("a substring collision must not expand the authority inventory");

    let extra_plan_test = copy_fixture();
    let registry = load_registry(extra_plan_test.path()).expect("fixture registry must parse");
    let test_plan = registry
        .tests
        .iter()
        .find(|test| test.id == "TST-FMT-001")
        .expect("authority test must exist")
        .source
        .clone();
    let test_plan_path = extra_plan_test.path().join(test_plan);
    let mut contents =
        fs::read_to_string(&test_plan_path).expect("test plan fixture must be readable");
    contents.push_str("\nInjected test TST-TRC-999.\n");
    fs::write(test_plan_path, contents).expect("test plan fixture must be writable");
    assert_semantic_contains(
        validate_registry(&registry, extra_plan_test.path()),
        "authoritative Test Plan ID set",
    );
}

// TST-TRC-014: a missing or stale generated matrix is rejected.
#[test]
fn matrix_drift_is_rejected() {
    let fixture = copy_fixture();
    assert_semantic_contains(check_workspace(fixture.path()), "generated matrix drift");
}

// TST-TRC-015: code test IDs must be registered; legacy bare tests remain inventory only.
#[test]
fn code_test_inventory_must_match_registry() {
    let fixture = copy_fixture();
    let registry = load_registry(fixture.path()).expect("fixture registry must parse");
    assert_eq!(registry.legacy_test_inventory[0].test_attributes, 174);
    let code_path = fixture.path().join(&registry.code_test_id_sources[0]);
    let mut source = fs::read_to_string(&code_path).expect("fixture code must be readable");
    source.push_str(&format!("\n// {}\n", ["TST", "EXTRA", "999"].join("-")));
    fs::write(code_path, source).expect("fixture code must be writable");
    assert_semantic_contains(
        validate_registry(&registry, fixture.path()),
        "comment test ID inventory",
    );

    let mut missing_locator = committed_registry();
    missing_locator
        .locators
        .retain(|locator| locator.test != "TST-REG-001");
    assert_semantic_contains(
        validate_registry(&missing_locator, &workspace_root()),
        "has no verified locator",
    );

    let mut planned_locator = committed_registry();
    let mut locator = planned_locator.locators[0].clone();
    locator.test = "TST-FMT-001".to_owned();
    planned_locator.locators.push(locator);
    assert_semantic_contains(
        validate_registry(&planned_locator, &workspace_root()),
        "PLANNED test",
    );

    let mut comment_only = committed_registry();
    comment_only.locators[0].symbol = "comment_only_marker".to_owned();
    assert_semantic_contains(
        validate_registry(&comment_only, &workspace_root()),
        "must resolve exactly once",
    );

    let mut non_test = committed_registry();
    non_test.locators[0].path = "tools/traceability-checker/src/lib.rs".to_owned();
    non_test.locators[0].symbol = "valid_test_id".to_owned();
    assert_semantic_contains(
        validate_registry(&non_test, &workspace_root()),
        "does not name a real #[test] function",
    );

    let missing_harness = copy_fixture();
    let harness_registry = load_registry(missing_harness.path()).expect("fixture registry parses");
    let harness = missing_harness.path().join(
        &harness_registry
            .locators
            .iter()
            .find(|locator| locator.kind == "trybuild")
            .expect("trybuild locator exists")
            .path,
    );
    fs::remove_file(harness).expect("temporary harness can be removed");
    assert_semantic_contains(
        validate_registry(&harness_registry, missing_harness.path()),
        "cannot read Rust test locator",
    );

    let missing_fixture = copy_fixture();
    let fixture_registry = load_registry(missing_fixture.path()).expect("fixture registry parses");
    let trybuild = fixture_registry
        .locators
        .iter()
        .find(|locator| locator.kind == "trybuild")
        .expect("trybuild locator exists");
    fs::remove_file(
        missing_fixture
            .path()
            .join(trybuild.fixture.as_ref().expect("fixture path exists")),
    )
    .expect("temporary trybuild fixture can be removed");
    assert_semantic_contains(
        validate_registry(&fixture_registry, missing_fixture.path()),
        "trybuild fixture does not exist",
    );

    let missing_stderr = copy_fixture();
    let stderr_registry = load_registry(missing_stderr.path()).expect("fixture registry parses");
    let trybuild = stderr_registry
        .locators
        .iter()
        .find(|locator| locator.kind == "trybuild")
        .expect("trybuild locator exists");
    fs::remove_file(
        missing_stderr
            .path()
            .join(trybuild.stderr.as_ref().expect("stderr path exists")),
    )
    .expect("temporary trybuild stderr can be removed");
    assert_semantic_contains(
        validate_registry(&stderr_registry, missing_stderr.path()),
        "trybuild stderr does not exist",
    );
}

// TST-TRC-016: CLI exit codes 0, 2, 3, and 4 are stable.
#[test]
fn cli_exit_codes_are_stable() {
    let binary = env!("CARGO_BIN_EXE_traceability-checker");
    let usage = Command::new(binary)
        .status()
        .expect("usage invocation must run");
    assert_eq!(usage.code(), Some(2));

    let valid = Command::new(binary)
        .args(["check", "--root"])
        .arg(workspace_root())
        .status()
        .expect("valid invocation must run");
    assert_eq!(valid.code(), Some(0));

    let empty = tempfile::tempdir().expect("empty fixture must be created");
    let operational = Command::new(binary)
        .args(["check", "--root"])
        .arg(empty.path())
        .status()
        .expect("operational failure invocation must run");
    assert_eq!(operational.code(), Some(3));

    let discovery = Command::new(binary)
        .arg("check")
        .current_dir(empty.path())
        .status()
        .expect("workspace discovery failure invocation must run");
    assert_eq!(discovery.code(), Some(3));

    let stale = copy_fixture();
    let semantic = Command::new(binary)
        .args(["check", "--root"])
        .arg(stale.path())
        .status()
        .expect("semantic failure invocation must run");
    assert_eq!(semantic.code(), Some(4));
}
