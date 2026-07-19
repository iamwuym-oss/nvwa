//! Machine-enforced requirement-to-test traceability for the NWB storage engine.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

pub const REGISTRY_PATH: &str = "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml";
const TEST_PLAN_PATH: &str = "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md";

const EXPECTED_SOURCES: &[&str] = &[
    "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Storage_Engine_Architecture_v2.0.md",
    "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md",
    "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md",
    "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Product_Support_Matrix_v1.0.md",
    "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md",
    TEST_PLAN_PATH,
    "crates/nwb-format/tests/registry_tests.rs",
    "crates/nwb-diagnostics/tests/diagnostics_contracts.rs",
    "tools/traceability-checker/tests/traceability_tests.rs",
    REGISTRY_PATH,
];

const EXPECTED_CODE_ID_SOURCES: &[&str] = &[
    "crates/nwb-format/tests/registry_tests.rs",
    "crates/nwb-diagnostics/tests/diagnostics_contracts.rs",
    "tools/traceability-checker/tests/traceability_tests.rs",
];

const EXPECTED_AUTHORITY_TEST_IDS: &[&str] = &[
    "TST-BLK-001",
    "TST-BLK-002",
    "TST-BLK-003",
    "TST-BLK-004",
    "TST-BLK-005",
    "TST-BLK-006",
    "TST-BLK-007",
    "TST-BLK-008",
    "TST-BLK-009",
    "TST-BMR-L-001",
    "TST-BMR-L-002",
    "TST-BMR-L-003",
    "TST-BMR-L-004",
    "TST-BMR-L-005",
    "TST-BMR-L-006",
    "TST-BMR-L-007",
    "TST-BMR-L-008",
    "TST-BMR-W-001",
    "TST-BMR-W-002",
    "TST-BMR-W-003",
    "TST-BMR-W-004",
    "TST-BMR-W-005",
    "TST-BMR-W-006",
    "TST-BMR-W-007",
    "TST-BMR-W-008",
    "TST-CRY-001",
    "TST-CRY-002",
    "TST-CRY-003",
    "TST-CRY-004",
    "TST-CRY-005",
    "TST-CRY-006",
    "TST-CRY-007",
    "TST-CRY-008",
    "TST-DIFF-001",
    "TST-DIFF-002",
    "TST-DIFF-003",
    "TST-DIFF-004",
    "TST-DIFF-005",
    "TST-DIFF-006",
    "TST-FAULT-001",
    "TST-FAULT-002",
    "TST-FAULT-003",
    "TST-FAULT-004",
    "TST-FAULT-005",
    "TST-FAULT-006",
    "TST-FAULT-007",
    "TST-FAULT-008",
    "TST-FILE-001",
    "TST-FILE-002",
    "TST-FILE-003",
    "TST-FMT-001",
    "TST-FMT-002",
    "TST-FMT-003",
    "TST-FMT-004",
    "TST-FMT-005",
    "TST-FMT-006",
    "TST-FMT-007",
    "TST-FMT-008",
    "TST-FMT-009",
    "TST-FMT-010",
    "TST-FMT-011",
    "TST-FMT-012",
    "TST-SAL-001",
    "TST-SAL-002",
    "TST-SAL-003",
    "TST-SAL-004",
    "TST-SAL-005",
    "TST-SEC-001",
    "TST-SEC-002",
    "TST-SEC-003",
    "TST-SEC-004",
    "TST-SEC-005",
    "TST-SEC-006",
    "TST-SEC-007",
    "TST-SEC-008",
    "TST-VER-001",
    "TST-VER-002",
    "TST-VER-003",
    "TST-VOL-001",
    "TST-VOL-002",
    "TST-VOL-003",
    "TST-VOL-004",
    "TST-VOL-005",
    "TST-VOL-006",
    "TST-VOL-007",
    "TST-VOL-008",
    "TST-VOL-009",
];

const EXPECTED_REGISTRY_PLANNED_TEST_IDS: &[&str] = &[
    "TST-FMT-013",
    "TST-FMT-014",
    "TST-FMT-015",
    "TST-PRV-001",
    "TST-PRV-002",
    "TST-PRV-003",
    "TST-PRV-004",
    "TST-PRV-005",
    "TST-PRV-006",
    "TST-PRV-007",
    "TST-PRV-008",
    "TST-PRV-009",
    "TST-PRV-010",
    "TST-PRV-011",
    "TST-PRV-012",
    "TST-PRV-013",
    "TST-PRV-014",
    "TST-PRV-015",
    "TST-PRV-016",
    "TST-REQ-001",
    "TST-REQ-002",
    "TST-REQ-003",
    "TST-REQ-004",
    "TST-REQ-005",
    "TST-REQ-006",
    "TST-REQ-007",
    "TST-REQ-008",
    "TST-REQ-009",
];

const EXPECTED_SOURCE_SCOPED_TEST_IDS: &[&str] = &[
    "TST-BLK-001",
    "TST-BLK-002",
    "TST-BLK-003",
    "TST-BLK-004",
    "TST-BLK-005",
    "TST-BLK-006",
    "TST-BLK-007",
    "TST-BLK-008",
    "TST-BLK-009",
    "TST-CRY-001",
    "TST-CRY-007",
    "TST-FAULT-005",
    "TST-FAULT-006",
    "TST-SEC-001",
    "TST-SEC-002",
    "TST-SEC-003",
    "TST-SEC-005",
    "TST-SEC-008",
    "TST-VOL-008",
];

#[derive(Debug)]
pub enum CheckError {
    Io(String),
    Schema(String),
    Semantic(String),
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "I/O error: {message}"),
            Self::Schema(message) => write!(formatter, "registry schema error: {message}"),
            Self::Semantic(message) => write!(formatter, "traceability violation: {message}"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: u32,
    pub matrix_path: String,
    pub source_allowlist: Vec<String>,
    pub code_test_id_sources: Vec<String>,
    pub legacy_test_inventory: Vec<LegacyInventory>,
    #[serde(rename = "requirement")]
    pub requirements: Vec<Requirement>,
    #[serde(rename = "test")]
    pub tests: Vec<TestCase>,
    #[serde(rename = "mapping")]
    pub mappings: Vec<Mapping>,
    #[serde(rename = "locator")]
    pub locators: Vec<Locator>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    pub source_id: String,
    pub title: String,
    pub priority: String,
    pub source: String,
    pub anchor: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCase {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub traceability: String,
    pub source: String,
    pub anchor: String,
    pub excerpt: String,
    pub scope_source: Option<String>,
    pub scope_anchor: Option<String>,
    pub scope_excerpt: Option<String>,
    pub scope_justification: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub requirement: String,
    pub test: String,
    pub role: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locator {
    pub test: String,
    pub kind: String,
    pub path: String,
    pub symbol: String,
    pub fixture: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyInventory {
    pub scope: String,
    pub test_attributes: u32,
    pub disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub requirements: usize,
    pub tests: usize,
    pub mappings: usize,
    pub p0_requirements: usize,
    pub planned_tests: usize,
    pub implemented_tests: usize,
    pub mapped_tests: usize,
    pub source_scoped_tests: usize,
}

pub fn check_workspace(root: &Path) -> Result<CheckReport, CheckError> {
    let registry = load_registry(root)?;
    let report = validate_registry(&registry, root)?;
    let expected = render_matrix(&registry, &report);
    let matrix_path = checked_relative_path(root, &registry.matrix_path)?;
    let current = fs::read_to_string(&matrix_path).map_err(|error| {
        CheckError::Semantic(format!(
            "generated matrix is missing or unreadable at {}: {error}",
            matrix_path.display()
        ))
    })?;
    if normalize_newlines(&current) != expected {
        return Err(CheckError::Semantic(format!(
            "generated matrix drift at {}; run `cargo run -p traceability-checker -- generate`",
            matrix_path.display()
        )));
    }
    Ok(report)
}

pub fn generate_matrix(root: &Path) -> Result<PathBuf, CheckError> {
    let registry = load_registry(root)?;
    let report = validate_registry(&registry, root)?;
    let matrix = render_matrix(&registry, &report);
    let path = checked_relative_path(root, &registry.matrix_path)?;
    fs::write(&path, matrix)
        .map_err(|error| CheckError::Io(format!("cannot write {}: {error}", path.display())))?;
    Ok(path)
}

pub fn load_registry(root: &Path) -> Result<Registry, CheckError> {
    let path = root.join(REGISTRY_PATH);
    let source = fs::read_to_string(&path)
        .map_err(|error| CheckError::Io(format!("cannot read {}: {error}", path.display())))?;
    toml::from_str(&source)
        .map_err(|error| CheckError::Schema(format!("{}: {error}", path.display())))
}

pub fn validate_registry(registry: &Registry, root: &Path) -> Result<CheckReport, CheckError> {
    if registry.schema_version != 1 {
        return semantic(format!(
            "unsupported schema_version {}; expected 1",
            registry.schema_version
        ));
    }
    if registry.matrix_path
        != "docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md"
    {
        return semantic("matrix_path is not the approved IMP-002 output".to_owned());
    }

    validate_exact_list(
        "source_allowlist",
        &registry.source_allowlist,
        EXPECTED_SOURCES,
    )?;
    validate_exact_list(
        "code_test_id_sources",
        &registry.code_test_id_sources,
        EXPECTED_CODE_ID_SOURCES,
    )?;
    validate_legacy_inventory(&registry.legacy_test_inventory)?;

    let expected_requirements = expected_requirements();
    validate_authority_requirement_inventory(registry, root)?;
    let mut requirements = BTreeMap::new();
    for requirement in &registry.requirements {
        if !valid_requirement_id(&requirement.id) {
            return semantic(format!("illegal requirement ID: {}", requirement.id));
        }
        if requirements
            .insert(requirement.id.clone(), requirement)
            .is_some()
        {
            return semantic(format!("duplicate requirement ID: {}", requirement.id));
        }
        if requirement.title.trim().is_empty() {
            return semantic(format!("requirement {} has an empty title", requirement.id));
        }
        validate_source_record(
            root,
            &registry.source_allowlist,
            &requirement.id,
            &requirement.source,
            &requirement.anchor,
            &requirement.excerpt,
        )?;
    }
    let actual_requirement_ids: BTreeSet<_> = requirements.keys().cloned().collect();
    let expected_requirement_ids: BTreeSet<_> = expected_requirements.keys().cloned().collect();
    if actual_requirement_ids != expected_requirement_ids {
        return semantic(format_set_difference(
            "requirement denominator",
            &expected_requirement_ids,
            &actual_requirement_ids,
        ));
    }
    let declared_p0 = requirements
        .values()
        .filter(|requirement| requirement.priority == "P0")
        .count();
    if declared_p0 == 0 {
        return semantic("registry contains zero P0 requirements".to_owned());
    }
    for (id, (priority, source_id, source)) in &expected_requirements {
        let Some(requirement) = requirements.get(id) else {
            return semantic(format!(
                "validated requirement {id} unexpectedly disappeared"
            ));
        };
        if &requirement.priority != priority
            || &requirement.source_id != source_id
            || &requirement.source != source
        {
            return semantic(format!(
                "requirement {id} must use priority={priority}, source_id={source_id}, source={source}"
            ));
        }
    }

    let mut tests = BTreeMap::new();
    for test in &registry.tests {
        if !valid_test_id(&test.id) {
            return semantic(format!("illegal test ID: {}", test.id));
        }
        if tests.insert(test.id.clone(), test).is_some() {
            return semantic(format!("duplicate test ID: {}", test.id));
        }
        if test.title.trim().is_empty() {
            return semantic(format!("test {} has an empty title", test.id));
        }
        if !matches!(
            test.kind.as_str(),
            "POSITIVE" | "NEGATIVE" | "FAULT" | "SECURITY"
        ) {
            return semantic(format!("test {} has illegal kind {}", test.id, test.kind));
        }
        if !matches!(test.status.as_str(), "PLANNED" | "IMPLEMENTED") {
            return semantic(format!(
                "test {} has illegal status {}",
                test.id, test.status
            ));
        }
        if !matches!(test.traceability.as_str(), "MAPPED" | "SOURCE_SCOPED") {
            return semantic(format!(
                "test {} has illegal traceability disposition {}",
                test.id, test.traceability
            ));
        }
        validate_source_record(
            root,
            &registry.source_allowlist,
            &test.id,
            &test.source,
            &test.anchor,
            &test.excerpt,
        )?;
    }
    validate_test_denominator(root, &tests)?;
    validate_locators(root, registry, &tests)?;
    validate_code_test_inventory(root, registry, &tests)?;

    let mut pairs = BTreeSet::new();
    let mut tests_to_requirements: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut requirements_to_roles: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for mapping in &registry.mappings {
        if !requirements.contains_key(&mapping.requirement) {
            return semantic(format!(
                "mapping references unknown requirement {}",
                mapping.requirement
            ));
        }
        if !tests.contains_key(&mapping.test) {
            return semantic(format!("mapping references unknown test {}", mapping.test));
        }
        if !matches!(
            mapping.role.as_str(),
            "POSITIVE" | "NEGATIVE" | "FAULT" | "SECURITY"
        ) {
            return semantic(format!(
                "mapping {} -> {} has illegal role {}",
                mapping.requirement, mapping.test, mapping.role
            ));
        }
        if mapping.rationale.trim().is_empty() {
            return semantic(format!(
                "mapping {} -> {} has an empty rationale",
                mapping.requirement, mapping.test
            ));
        }
        if !pairs.insert((mapping.requirement.as_str(), mapping.test.as_str())) {
            return semantic(format!(
                "duplicate mapping: {} -> {}",
                mapping.requirement, mapping.test
            ));
        }
        tests_to_requirements
            .entry(mapping.test.as_str())
            .or_default()
            .insert(mapping.requirement.as_str());
        requirements_to_roles
            .entry(mapping.requirement.as_str())
            .or_default()
            .insert(mapping.role.as_str());
    }
    validate_exact_mappings(registry)?;
    let (mapped_tests, source_scoped_tests) = validate_test_dispositions(
        root,
        &registry.source_allowlist,
        &tests,
        &tests_to_requirements,
    )?;

    let p0_count = requirements
        .values()
        .filter(|requirement| requirement.priority == "P0")
        .count();
    for requirement in requirements.values() {
        let roles = requirements_to_roles
            .get(requirement.id.as_str())
            .cloned()
            .unwrap_or_default();
        if requirement.priority == "P0" {
            if !roles.contains("POSITIVE") {
                return semantic(format!(
                    "P0 requirement {} lacks a POSITIVE test",
                    requirement.id
                ));
            }
            if !roles
                .iter()
                .any(|kind| matches!(*kind, "NEGATIVE" | "FAULT" | "SECURITY"))
            {
                return semantic(format!(
                    "P0 requirement {} lacks a NEGATIVE, FAULT, or SECURITY test",
                    requirement.id
                ));
            }
        } else if roles.is_empty() {
            return semantic(format!("P1 requirement {} has no test", requirement.id));
        }
    }

    let implemented_tests = tests
        .values()
        .filter(|test| test.status == "IMPLEMENTED")
        .count();
    Ok(CheckReport {
        requirements: requirements.len(),
        tests: tests.len(),
        mappings: pairs.len(),
        p0_requirements: p0_count,
        planned_tests: tests.len() - implemented_tests,
        implemented_tests,
        mapped_tests,
        source_scoped_tests,
    })
}

pub fn render_matrix(registry: &Registry, report: &CheckReport) -> String {
    let mut test_map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for mapping in &registry.mappings {
        test_map
            .entry(mapping.requirement.as_str())
            .or_default()
            .push(mapping.test.as_str());
    }
    for tests in test_map.values_mut() {
        tests.sort_unstable();
        tests.dedup();
    }

    let mut requirements: Vec<_> = registry.requirements.iter().collect();
    requirements.sort_by(|left, right| left.id.cmp(&right.id));
    let mut tests: Vec<_> = registry.tests.iter().collect();
    tests.sort_by(|left, right| left.id.cmp(&right.id));

    let mut output = String::new();
    output.push_str("# Nüwa NWB 需求-测试追溯矩阵 v1.0\n\n");
    output.push_str("> 本文档由 `traceability-checker` 从 `Nuwa_NWB_Traceability_Registry_v1.0.toml` 确定性生成。请勿手工编辑。\n\n");
    output.push_str("## 1. 覆盖摘要\n\n");
    output.push_str("| 指标 | 数量 |\n|---|---:|\n");
    output.push_str(&format!("| 正式需求 | {} |\n", report.requirements));
    output.push_str(&format!("| P0需求 | {} |\n", report.p0_requirements));
    output.push_str(&format!(
        "| P1需求 | {} |\n",
        report.requirements - report.p0_requirements
    ));
    output.push_str(&format!("| 正式测试 | {} |\n", report.tests));
    output.push_str(&format!("| 已实现测试 | {} |\n", report.implemented_tests));
    output.push_str(&format!("| 计划测试 | {} |\n", report.planned_tests));
    output.push_str(&format!("| 已映射测试 | {} |\n", report.mapped_tests));
    output.push_str(&format!(
        "| Source-scoped测试 | {} |\n",
        report.source_scoped_tests
    ));
    output.push_str(&format!("| 唯一映射 | {} |\n\n", report.mappings));

    output.push_str("## 2. 需求到测试\n\n");
    output.push_str(
        "| Requirement | Priority | Source ID | Requirement | Tests |\n|---|---|---|---|---|\n",
    );
    for requirement in requirements {
        let mapped = test_map
            .get(requirement.id.as_str())
            .map(|values| values.join(", "))
            .unwrap_or_default();
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            requirement.id,
            requirement.priority,
            requirement.source_id,
            escape_cell(&requirement.title),
            mapped
        ));
    }

    output.push_str("\n## 3. 测试登记\n\n");
    output.push_str(
        "| Test | Kind | Status | Traceability | Test case | Source |\n|---|---|---|---|---|---|\n",
    );
    for test in tests {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            test.id,
            test.kind,
            test.status,
            test.traceability,
            escape_cell(&test.title),
            test.source
        ));
    }

    output.push_str("\n## 4. Source-scoped测试库存\n\n");
    output.push_str("| Test | Authority source | Anchor | Justification |\n|---|---|---|---|\n");
    for test in registry
        .tests
        .iter()
        .filter(|test| test.traceability == "SOURCE_SCOPED")
    {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            test.id,
            test.scope_source.as_deref().unwrap_or_default(),
            escape_cell(test.scope_anchor.as_deref().unwrap_or_default()),
            escape_cell(test.scope_justification.as_deref().unwrap_or_default())
        ));
    }

    output.push_str("\n## 5. 逐边证明语义\n\n");
    output.push_str("| Requirement | Test | Role | Rationale |\n|---|---|---|---|\n");
    let mut mappings: Vec<_> = registry.mappings.iter().collect();
    mappings.sort_by(|left, right| {
        (&left.requirement, &left.test).cmp(&(&right.requirement, &right.test))
    });
    for mapping in mappings {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            mapping.requirement,
            mapping.test,
            mapping.role,
            escape_cell(&mapping.rationale)
        ));
    }

    output.push_str("\n## 6. 旧测试库存（非正式追溯分母）\n\n");
    output.push_str("| Scope | `#[test]` count | Disposition |\n|---|---:|---|\n");
    let mut inventory: Vec<_> = registry.legacy_test_inventory.iter().collect();
    inventory.sort_by(|left, right| left.scope.cmp(&right.scope));
    for item in inventory {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_cell(&item.scope),
            item.test_attributes,
            escape_cell(&item.disposition)
        ));
    }
    output
}

fn validate_test_denominator(
    root: &Path,
    tests: &BTreeMap<String, &TestCase>,
) -> Result<(), CheckError> {
    let plan = fs::read_to_string(root.join(TEST_PLAN_PATH)).map_err(|error| {
        CheckError::Semantic(format!("cannot read authoritative test plan: {error}"))
    })?;
    let authority_ids: BTreeSet<_> = extract_test_ids(&plan);
    let frozen_authority_ids: BTreeSet<_> = EXPECTED_AUTHORITY_TEST_IDS
        .iter()
        .map(|id| (*id).to_owned())
        .collect();
    if authority_ids != frozen_authority_ids {
        return semantic(format_set_difference(
            "authoritative Test Plan ID set",
            &frozen_authority_ids,
            &authority_ids,
        ));
    }

    let expected_registry: BTreeSet<_> = (1..=7)
        .map(|number| format!("TST-REG-{number:03}"))
        .collect();
    let expected_traceability: BTreeSet<_> = (1..=16)
        .map(|number| format!("TST-TRC-{number:03}"))
        .collect();
    let expected_diagnostics: BTreeSet<_> = (1..=6)
        .map(|number| format!("TST-ERR-{number:03}"))
        .collect();
    let registry_planned: BTreeSet<_> = EXPECTED_REGISTRY_PLANNED_TEST_IDS
        .iter()
        .map(|id| (*id).to_owned())
        .collect();
    let mut planned_ids = authority_ids;
    planned_ids.extend(registry_planned);
    let mut expected = planned_ids.clone();
    expected.extend(expected_registry.iter().cloned());
    expected.extend(expected_traceability.iter().cloned());
    expected.extend(expected_diagnostics.iter().cloned());
    let actual: BTreeSet<_> = tests.keys().cloned().collect();
    if actual != expected {
        return semantic(format_set_difference(
            "test denominator",
            &expected,
            &actual,
        ));
    }
    if actual.len() != 144 {
        return semantic(format!(
            "formal test denominator is {}; expected 144",
            actual.len()
        ));
    }

    for (id, test) in tests {
        let expected_status = if planned_ids.contains(id) {
            "PLANNED"
        } else {
            "IMPLEMENTED"
        };
        if test.status != expected_status {
            return semantic(format!(
                "test {id} must have status {expected_status}, not {}",
                test.status
            ));
        }
        if EXPECTED_REGISTRY_PLANNED_TEST_IDS.contains(&id.as_str()) && test.source != REGISTRY_PATH
        {
            return semantic(format!(
                "registry-defined planned test {id} must use the registry as its source"
            ));
        }
    }
    Ok(())
}

fn validate_authority_requirement_inventory(
    registry: &Registry,
    root: &Path,
) -> Result<(), CheckError> {
    let architecture_ids: BTreeSet<_> = (1..=16).map(|number| format!("P-{number:02}")).collect();
    validate_native_requirement_inventory(
        registry,
        root,
        EXPECTED_SOURCES[0],
        "P",
        2,
        &architecture_ids,
    )?;

    let format_ids: BTreeSet<_> = (1..=10).map(|number| format!("FMT-{number:03}")).collect();
    validate_native_requirement_inventory(
        registry,
        root,
        EXPECTED_SOURCES[1],
        "FMT",
        3,
        &format_ids,
    )?;

    let provider_ids: BTreeSet<_> = (1..=8).map(|number| format!("PRV-{number:03}")).collect();
    validate_native_requirement_inventory(
        registry,
        root,
        EXPECTED_SOURCES[2],
        "PRV",
        3,
        &provider_ids,
    )
}

fn validate_native_requirement_inventory(
    registry: &Registry,
    root: &Path,
    source: &str,
    prefix: &str,
    digits: usize,
    approved: &BTreeSet<String>,
) -> Result<(), CheckError> {
    let path = checked_relative_path(root, source)?;
    let contents = fs::read_to_string(&path).map_err(|error| {
        CheckError::Semantic(format!(
            "cannot read authoritative {prefix} requirement source {}: {error}",
            path.display()
        ))
    })?;
    let authority_ids = extract_native_ids(&normalize_newlines(&contents), prefix, digits);
    if authority_ids != *approved {
        return semantic(format_set_difference(
            &format!("authoritative {prefix} ID inventory"),
            approved,
            &authority_ids,
        ));
    }

    let registry_ids: BTreeSet<_> = registry
        .requirements
        .iter()
        .filter(|requirement| requirement.source == source)
        .map(|requirement| requirement.source_id.clone())
        .collect();
    if registry_ids != *approved {
        return semantic(format_set_difference(
            &format!("registry {prefix} source ID inventory"),
            approved,
            &registry_ids,
        ));
    }
    if registry_ids != authority_ids {
        return semantic(format_set_difference(
            &format!("registry-to-authority {prefix} ID inventory"),
            &authority_ids,
            &registry_ids,
        ));
    }
    Ok(())
}

fn validate_code_test_inventory(
    root: &Path,
    registry: &Registry,
    tests: &BTreeMap<String, &TestCase>,
) -> Result<(), CheckError> {
    let mut code_ids = BTreeSet::new();
    for relative in &registry.code_test_id_sources {
        let path = checked_relative_path(root, relative)?;
        let source = fs::read_to_string(&path).map_err(|error| {
            CheckError::Semantic(format!(
                "cannot read code test source {}: {error}",
                path.display()
            ))
        })?;
        code_ids.extend(extract_comment_test_ids(&source));
    }
    let registered: BTreeSet<_> = tests.keys().cloned().collect();
    if !code_ids.is_subset(&registered) {
        return semantic(format_set_difference(
            "comment test ID inventory",
            &registered,
            &code_ids,
        ));
    }
    Ok(())
}

fn validate_locators(
    root: &Path,
    registry: &Registry,
    tests: &BTreeMap<String, &TestCase>,
) -> Result<(), CheckError> {
    let mut by_test: BTreeMap<&str, Vec<&Locator>> = BTreeMap::new();
    let mut unique = BTreeSet::new();
    for locator in &registry.locators {
        let Some(test) = tests.get(&locator.test) else {
            return semantic(format!("locator references unknown test {}", locator.test));
        };
        if test.status != "IMPLEMENTED" {
            return semantic(format!(
                "PLANNED test {} must not have an implementation locator",
                locator.test
            ));
        }
        let key = (
            locator.test.as_str(),
            locator.kind.as_str(),
            locator.path.as_str(),
            locator.symbol.as_str(),
            locator.fixture.as_deref(),
            locator.stderr.as_deref(),
        );
        if !unique.insert(key) {
            return semantic(format!("duplicate locator for test {}", locator.test));
        }
        match locator.kind.as_str() {
            "rust_test" => {
                if locator.fixture.is_some() || locator.stderr.is_some() {
                    return semantic(format!(
                        "rust_test locator for {} must not define fixture or stderr",
                        locator.test
                    ));
                }
                validate_rust_test(root, &locator.path, &locator.symbol)?;
            }
            "trybuild" => validate_trybuild_locator(root, locator)?,
            other => {
                return semantic(format!(
                    "locator for {} has illegal kind {other}",
                    locator.test
                ));
            }
        }
        by_test
            .entry(locator.test.as_str())
            .or_default()
            .push(locator);
    }
    for test in tests.values() {
        let has_locator = by_test.contains_key(test.id.as_str());
        if test.status == "IMPLEMENTED" && !has_locator {
            return semantic(format!(
                "IMPLEMENTED test {} has no verified locator",
                test.id
            ));
        }
        if test.status == "PLANNED" && has_locator {
            return semantic(format!(
                "PLANNED test {} must not have an implementation locator",
                test.id
            ));
        }
    }
    Ok(())
}

fn validate_rust_test(root: &Path, relative: &str, symbol: &str) -> Result<String, CheckError> {
    if !valid_rust_symbol(symbol) {
        return semantic(format!("invalid Rust test symbol: {symbol}"));
    }
    let path = checked_relative_path(root, relative)?;
    let source = fs::read_to_string(&path).map_err(|error| {
        CheckError::Semantic(format!(
            "cannot read Rust test locator {}: {error}",
            path.display()
        ))
    })?;
    let source = normalize_newlines(&source);
    let signature = format!("fn {symbol}(");
    let matches: Vec<_> = source.match_indices(&signature).collect();
    if matches.len() != 1 {
        return semantic(format!(
            "Rust test locator {relative}::{symbol} must resolve exactly once"
        ));
    }
    let position = matches[0].0;
    let previous = source[..position]
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim);
    if previous != Some("#[test]") {
        return semantic(format!(
            "Rust locator {relative}::{symbol} does not name a real #[test] function"
        ));
    }
    Ok(source[position..].to_owned())
}

fn validate_trybuild_locator(root: &Path, locator: &Locator) -> Result<(), CheckError> {
    let Some(fixture) = locator.fixture.as_deref() else {
        return semantic(format!(
            "trybuild locator for {} is missing fixture",
            locator.test
        ));
    };
    let Some(stderr) = locator.stderr.as_deref() else {
        return semantic(format!(
            "trybuild locator for {} is missing stderr",
            locator.test
        ));
    };
    let body = validate_rust_test(root, &locator.path, &locator.symbol)?;
    let harness_path = Path::new(&locator.path);
    let Some(crate_root) = harness_path.parent().and_then(Path::parent) else {
        return semantic(format!(
            "trybuild harness path has no crate root: {}",
            locator.path
        ));
    };
    let fixture_path = Path::new(fixture);
    let Ok(fixture_reference) = fixture_path.strip_prefix(crate_root) else {
        return semantic(format!(
            "trybuild fixture {fixture} is outside harness crate {}",
            crate_root.display()
        ));
    };
    let fixture_reference = fixture_reference.to_string_lossy().replace('\\', "/");
    let expected_call = format!("compile_fail(\"{fixture_reference}\")");
    if !body.contains(&expected_call) {
        return semantic(format!(
            "trybuild harness {}::{} does not reference fixture {fixture_reference}",
            locator.path, locator.symbol
        ));
    }
    let fixture_absolute = checked_relative_path(root, fixture)?;
    if !fixture_absolute.is_file() {
        return semantic(format!("trybuild fixture does not exist: {fixture}"));
    }
    let stderr_absolute = checked_relative_path(root, stderr)?;
    if !stderr_absolute.is_file() {
        return semantic(format!("trybuild stderr does not exist: {stderr}"));
    }
    if fixture_path.with_extension("stderr") != Path::new(stderr) {
        return semantic(format!(
            "trybuild stderr {stderr} does not match fixture {fixture}"
        ));
    }
    Ok(())
}

fn valid_rust_symbol(symbol: &str) -> bool {
    let mut bytes = symbol.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn validate_source_record(
    root: &Path,
    allowlist: &[String],
    id: &str,
    source: &str,
    anchor: &str,
    excerpt: &str,
) -> Result<(), CheckError> {
    if !allowlist.iter().any(|allowed| allowed == source) {
        return semantic(format!("{id} uses non-allowlisted source {source}"));
    }
    if anchor.trim().is_empty() || excerpt.trim().is_empty() {
        return semantic(format!("{id} has an empty source anchor or excerpt"));
    }
    let path = checked_relative_path(root, source)?;
    let contents = fs::read_to_string(&path).map_err(|error| {
        CheckError::Semantic(format!(
            "cannot read registered source {}: {error}",
            path.display()
        ))
    })?;
    let contents = normalize_newlines(&contents);
    let anchor_count = contents.matches(anchor).count();
    let excerpt_count = contents.matches(excerpt).count();
    if anchor_count != 1 || excerpt_count != 1 {
        return semantic(format!(
            "{id} source anchor/excerpt must each match exactly once in {source}; anchor={anchor_count}, excerpt={excerpt_count}"
        ));
    }
    Ok(())
}

fn validate_exact_list(name: &str, actual: &[String], expected: &[&str]) -> Result<(), CheckError> {
    let actual_set: BTreeSet<_> = actual.iter().cloned().collect();
    if actual_set.len() != actual.len() {
        return semantic(format!("{name} contains duplicate paths"));
    }
    let expected_set: BTreeSet<_> = expected.iter().map(|value| (*value).to_owned()).collect();
    if actual_set != expected_set {
        return semantic(format_set_difference(name, &expected_set, &actual_set));
    }
    Ok(())
}

fn validate_legacy_inventory(inventory: &[LegacyInventory]) -> Result<(), CheckError> {
    if inventory.is_empty() {
        return semantic("legacy test inventory is empty".to_owned());
    }
    let mut scopes = BTreeSet::new();
    for item in inventory {
        if item.scope.trim().is_empty()
            || item.disposition.trim().is_empty()
            || item.test_attributes == 0
        {
            return semantic("legacy test inventory contains an empty or zero entry".to_owned());
        }
        if !scopes.insert(&item.scope) {
            return semantic(format!(
                "duplicate legacy test inventory scope: {}",
                item.scope
            ));
        }
    }
    Ok(())
}

fn validate_test_dispositions(
    root: &Path,
    allowlist: &[String],
    tests: &BTreeMap<String, &TestCase>,
    tests_to_requirements: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<(usize, usize), CheckError> {
    let expected_source_scoped: BTreeSet<_> = EXPECTED_SOURCE_SCOPED_TEST_IDS
        .iter()
        .map(|id| (*id).to_owned())
        .collect();
    let actual_source_scoped: BTreeSet<_> = tests
        .values()
        .filter(|test| test.traceability == "SOURCE_SCOPED")
        .map(|test| test.id.clone())
        .collect();
    if actual_source_scoped != expected_source_scoped {
        return semantic(format_set_difference(
            "SOURCE_SCOPED test set",
            &expected_source_scoped,
            &actual_source_scoped,
        ));
    }

    let authority_sources: BTreeSet<_> = EXPECTED_SOURCES[..6].iter().copied().collect();
    let mut mapped = 0;
    let mut scoped = 0;
    for test in tests.values() {
        let has_edge = tests_to_requirements.contains_key(test.id.as_str());
        let scope_fields_absent = test.scope_source.is_none()
            && test.scope_anchor.is_none()
            && test.scope_excerpt.is_none()
            && test.scope_justification.is_none();
        match test.traceability.as_str() {
            "MAPPED" => {
                if !has_edge {
                    return semantic(format!("MAPPED test {} has no requirement edge", test.id));
                }
                if !scope_fields_absent {
                    return semantic(format!(
                        "MAPPED test {} must not define scope metadata",
                        test.id
                    ));
                }
                mapped += 1;
            }
            "SOURCE_SCOPED" => {
                if has_edge {
                    return semantic(format!(
                        "SOURCE_SCOPED test {} must have zero requirement edges",
                        test.id
                    ));
                }
                let (Some(source), Some(anchor), Some(excerpt), Some(justification)) = (
                    test.scope_source.as_deref(),
                    test.scope_anchor.as_deref(),
                    test.scope_excerpt.as_deref(),
                    test.scope_justification.as_deref(),
                ) else {
                    return semantic(format!(
                        "SOURCE_SCOPED test {} is missing authority scope metadata",
                        test.id
                    ));
                };
                if !authority_sources.contains(source) {
                    return semantic(format!(
                        "SOURCE_SCOPED test {} uses non-authoritative scope source {source}",
                        test.id
                    ));
                }
                if justification.trim().is_empty() {
                    return semantic(format!(
                        "SOURCE_SCOPED test {} has an empty scope justification",
                        test.id
                    ));
                }
                validate_source_record(
                    root,
                    allowlist,
                    &format!("{} scope", test.id),
                    source,
                    anchor,
                    excerpt,
                )?;
                scoped += 1;
            }
            _ => unreachable!("traceability disposition was validated earlier"),
        }
    }
    if mapped != 125 || scoped != 19 {
        return semantic(format!(
            "test disposition counts mismatch; mapped={mapped}, source_scoped={scoped}"
        ));
    }
    Ok((mapped, scoped))
}

fn validate_exact_mappings(registry: &Registry) -> Result<(), CheckError> {
    let exact: &[(&str, &[(&str, &str)])] = &[
        (
            "REQ-001",
            &[("TST-REQ-001", "POSITIVE"), ("TST-REQ-002", "NEGATIVE")],
        ),
        ("REQ-006", &[("TST-REQ-003", "POSITIVE")]),
        (
            "REQ-009",
            &[
                ("TST-FMT-004", "POSITIVE"),
                ("TST-FMT-003", "NEGATIVE"),
                ("TST-FMT-005", "NEGATIVE"),
                ("TST-SEC-004", "NEGATIVE"),
                ("TST-SEC-007", "SECURITY"),
            ],
        ),
        (
            "REQ-010",
            &[("TST-REQ-004", "POSITIVE"), ("TST-REQ-005", "NEGATIVE")],
        ),
        (
            "REQ-013",
            &[("TST-REQ-006", "POSITIVE"), ("TST-REQ-007", "NEGATIVE")],
        ),
        (
            "REQ-014",
            &[("TST-REQ-008", "POSITIVE"), ("TST-REQ-009", "NEGATIVE")],
        ),
        (
            "REQ-019",
            &[
                ("TST-ERR-001", "POSITIVE"),
                ("TST-ERR-002", "NEGATIVE"),
                ("TST-ERR-003", "POSITIVE"),
                ("TST-ERR-004", "SECURITY"),
                ("TST-ERR-005", "SECURITY"),
                ("TST-ERR-006", "SECURITY"),
            ],
        ),
        (
            "FMT-001",
            &[("TST-FMT-001", "POSITIVE"), ("TST-FMT-013", "NEGATIVE")],
        ),
        (
            "FMT-003",
            &[("TST-FMT-006", "POSITIVE"), ("TST-FMT-014", "NEGATIVE")],
        ),
        (
            "FMT-005",
            &[("TST-VOL-001", "POSITIVE"), ("TST-FMT-015", "NEGATIVE")],
        ),
        (
            "FMT-007",
            &[
                ("TST-FILE-002", "POSITIVE"),
                ("TST-DIFF-004", "NEGATIVE"),
                ("TST-FMT-005", "NEGATIVE"),
                ("TST-FMT-011", "NEGATIVE"),
                ("TST-SEC-007", "SECURITY"),
            ],
        ),
        (
            "FMT-009",
            &[
                ("TST-CRY-002", "POSITIVE"),
                ("TST-CRY-003", "NEGATIVE"),
                ("TST-CRY-004", "SECURITY"),
            ],
        ),
        (
            "PRV-001",
            &[("TST-PRV-001", "POSITIVE"), ("TST-PRV-002", "NEGATIVE")],
        ),
        (
            "PRV-002",
            &[("TST-PRV-003", "POSITIVE"), ("TST-PRV-004", "NEGATIVE")],
        ),
        (
            "PRV-003",
            &[("TST-PRV-005", "POSITIVE"), ("TST-PRV-006", "NEGATIVE")],
        ),
        (
            "PRV-004",
            &[
                ("TST-PRV-007", "POSITIVE"),
                ("TST-PRV-008", "SECURITY"),
                ("TST-SEC-006", "SECURITY"),
            ],
        ),
        (
            "PRV-005",
            &[
                ("TST-PRV-009", "POSITIVE"),
                ("TST-PRV-010", "FAULT"),
                ("TST-SEC-006", "FAULT"),
            ],
        ),
        (
            "PRV-006",
            &[("TST-PRV-011", "POSITIVE"), ("TST-PRV-012", "NEGATIVE")],
        ),
        (
            "PRV-007",
            &[("TST-PRV-013", "POSITIVE"), ("TST-PRV-014", "SECURITY")],
        ),
        (
            "PRV-008",
            &[("TST-PRV-015", "POSITIVE"), ("TST-PRV-016", "SECURITY")],
        ),
    ];
    for (requirement, expected_edges) in exact {
        let actual: BTreeSet<_> = registry
            .mappings
            .iter()
            .filter(|mapping| mapping.requirement == *requirement)
            .map(|mapping| (mapping.test.as_str(), mapping.role.as_str()))
            .collect();
        let expected: BTreeSet<_> = expected_edges.iter().copied().collect();
        if actual != expected {
            return semantic(format!(
                "exact mapping set for {requirement} mismatch; expected={expected:?}, actual={actual:?}"
            ));
        }
    }
    Ok(())
}

fn expected_requirements() -> BTreeMap<String, (String, String, String)> {
    let architecture = EXPECTED_SOURCES[0];
    let format = EXPECTED_SOURCES[1];
    let provider = EXPECTED_SOURCES[2];
    let implementation = EXPECTED_SOURCES[4];
    let p1: BTreeSet<_> = [6, 11, 12].into_iter().collect();
    let mut expected = BTreeMap::new();
    for number in 1..=16 {
        expected.insert(
            format!("REQ-{number:03}"),
            (
                if p1.contains(&number) { "P1" } else { "P0" }.to_owned(),
                format!("P-{number:02}"),
                architecture.to_owned(),
            ),
        );
    }
    expected.insert(
        "REQ-017".to_owned(),
        (
            "P0".to_owned(),
            "IMP-001".to_owned(),
            implementation.to_owned(),
        ),
    );
    expected.insert(
        "REQ-018".to_owned(),
        (
            "P0".to_owned(),
            "IMP-002".to_owned(),
            implementation.to_owned(),
        ),
    );
    expected.insert(
        "REQ-019".to_owned(),
        (
            "P0".to_owned(),
            "IMP-003".to_owned(),
            implementation.to_owned(),
        ),
    );
    for number in 1..=10 {
        let id = format!("FMT-{number:03}");
        expected.insert(id.clone(), ("P0".to_owned(), id, format.to_owned()));
    }
    for number in 1..=8 {
        let id = format!("PRV-{number:03}");
        expected.insert(id.clone(), ("P0".to_owned(), id, provider.to_owned()));
    }
    expected
}

fn checked_relative_path(root: &Path, relative: &str) -> Result<PathBuf, CheckError> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return semantic(format!(
            "path must be a normalized repository-relative path: {relative}"
        ));
    }
    Ok(root.join(path))
}

fn valid_requirement_id(id: &str) -> bool {
    let Some((prefix, number)) = id.rsplit_once('-') else {
        return false;
    };
    matches!(prefix, "REQ" | "FMT" | "PRV" | "SEC" | "REL" | "BMR")
        && number.len() == 3
        && number.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn valid_test_id(id: &str) -> bool {
    let parts: Vec<_> = id.split('-').collect();
    if parts.len() < 3 || parts[0] != "TST" {
        return false;
    }
    let number = parts[parts.len() - 1];
    number.len() == 3
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && !parts[1].is_empty()
        && parts[1].as_bytes()[0].is_ascii_uppercase()
        && parts[1]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        && parts[2..parts.len() - 1].iter().all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        })
}

fn extract_test_ids(source: &str) -> BTreeSet<String> {
    source
        .split(|character: char| {
            !(character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-')
        })
        .filter(|token| valid_test_id(token))
        .map(str::to_owned)
        .collect()
}

fn extract_native_ids(source: &str, prefix: &str, digits: usize) -> BTreeSet<String> {
    source
        .split(|character: char| {
            !(character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-')
        })
        .filter(|token| {
            let Some((candidate_prefix, number)) = token.split_once('-') else {
                return false;
            };
            candidate_prefix == prefix
                && number.len() == digits
                && number.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(str::to_owned)
        .collect()
}

fn extract_comment_test_ids(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter(|line| line.trim_start().starts_with("//"))
        .flat_map(extract_test_ids)
        .collect()
}

fn normalize_newlines(value: &str) -> String {
    value
        .strip_prefix('\u{feff}')
        .unwrap_or(value)
        .replace("\r\n", "\n")
}

fn format_set_difference(
    name: &str,
    expected: &BTreeSet<String>,
    actual: &BTreeSet<String>,
) -> String {
    let missing: Vec<_> = expected.difference(actual).cloned().collect();
    let unexpected: Vec<_> = actual.difference(expected).cloned().collect();
    format!("{name} mismatch; missing={missing:?}, unexpected={unexpected:?}")
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn semantic<T>(message: String) -> Result<T, CheckError> {
    Err(CheckError::Semantic(message))
}
