//! Integration tests for the NWB Format Registry.
//!
//! Coverage:
//!
//! | ID        | Target                                          |
//! |-----------|-------------------------------------------------|
//! | TST-REG-001 | All 18 RecordType variants exist and are matchable |
//! | TST-REG-002 | Duplicate discriminant rejected by compiler (see comment) |
//! | TST-REG-003 | 5 Feature bits — no two share the same position  |
//! | TST-REG-004 | All bit positions within 0..63                  |
//! | TST-REG-005 | BackupKind / PlatformHint have exact values      |
//! | TST-REG-006 | Registry snapshot matches expected output         |
//! | TST-REG-007 | TOML data files correspond to enum variants       |

use std::collections::HashSet;
use std::path::Path;

use nwb_format::registry;

// ---------------------------------------------------------------------------
// TST-REG-001: All 18 RecordType variants exist and can be matched
// ---------------------------------------------------------------------------

#[test]
fn test_record_type_all_variants_matchable() {
    let cases: &[(registry::record_type::RecordType, u16, &str)] = &[
        (
            registry::record_type::RecordType::Invalid,
            0x0000,
            "Invalid",
        ),
        (
            registry::record_type::RecordType::Manifest,
            0x0001,
            "Manifest",
        ),
        (
            registry::record_type::RecordType::VolumeSetManifest,
            0x0002,
            "VolumeSetManifest",
        ),
        (
            registry::record_type::RecordType::VolumeFooter,
            0x0003,
            "VolumeFooter",
        ),
        (
            registry::record_type::RecordType::Tombstone,
            0x0004,
            "Tombstone",
        ),
        (
            registry::record_type::RecordType::DataChunk,
            0x0101,
            "DataChunk",
        ),
        (
            registry::record_type::RecordType::ZeroRun,
            0x0102,
            "ZeroRun",
        ),
        (
            registry::record_type::RecordType::HoleRun,
            0x0103,
            "HoleRun",
        ),
        (
            registry::record_type::RecordType::FileExtent,
            0x0104,
            "FileExtent",
        ),
        (
            registry::record_type::RecordType::DiskExtent,
            0x0105,
            "DiskExtent",
        ),
        (
            registry::record_type::RecordType::DiffOperation,
            0x0106,
            "DiffOperation",
        ),
        (
            registry::record_type::RecordType::CatalogPage,
            0x0201,
            "CatalogPage",
        ),
        (
            registry::record_type::RecordType::ChunkIndexPage,
            0x0202,
            "ChunkIndexPage",
        ),
        (
            registry::record_type::RecordType::ErrorRecord,
            0x0301,
            "ErrorRecord",
        ),
        (
            registry::record_type::RecordType::FileEntry,
            0x0401,
            "FileEntry",
        ),
        (
            registry::record_type::RecordType::PartitionLayout,
            0x0402,
            "PartitionLayout",
        ),
        (
            registry::record_type::RecordType::BmrArtifact,
            0x0403,
            "BmrArtifact",
        ),
        (
            registry::record_type::RecordType::KeySlot,
            0x0501,
            "KeySlot",
        ),
    ];

    assert_eq!(cases.len(), 18, "expected exactly 18 record type variants");

    for (variant, expected_id, expected_name) in cases {
        assert_eq!(
            *variant as u16, *expected_id,
            "RecordType discriminant mismatch for {expected_name}"
        );
        assert_eq!(
            variant.to_string(),
            *expected_name,
            "RecordType Display mismatch for {expected_name}"
        );
    }
}

// ---------------------------------------------------------------------------
// TST-REG-002: Duplicate discriminant is a compile-time error
//
// The compiler rejects duplicate discriminants in `#[repr(u16)]` enums at
// compile time.  To verify, uncomment the line below and run `cargo build`:
//
// ```compile_fail
// pub enum RecordTypeDup { A = 0x0001, B = 0x0001 }
// ```
//
// Since we cannot depend on `trybuild` (no dev-dependency approval), this
// property is proven by: (a) the compiler inherent enum discriminant
// check, and (b) manual verification in the build step.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// TST-REG-003 + TST-REG-004: Feature bit uniqueness and range
// ---------------------------------------------------------------------------

#[test]
fn test_feature_bits_no_duplicates_within_range() {
    let bits: &[(u8, &str)] = &[
        (registry::feature_bit::COMPRESSION_ZSTD, "COMPRESSION_ZSTD"),
        (
            registry::feature_bit::ENCRYPTION_AES256_GCM,
            "ENCRYPTION_AES256_GCM",
        ),
        (registry::feature_bit::VOLUME_SET, "VOLUME_SET"),
        (registry::feature_bit::CHECKPOINT, "CHECKPOINT"),
        (registry::feature_bit::BMR_METADATA, "BMR_METADATA"),
    ];

    assert_eq!(bits.len(), 5, "expected exactly 5 allocated feature bits");

    let mut seen = HashSet::new();
    for &(pos, name) in bits {
        assert!(
            pos < 64,
            "{name} position {pos} exceeds maximum bit index 63"
        );
        assert!(
            seen.insert(pos),
            "{name} position {pos} duplicates an earlier feature bit"
        );
    }
}

// ---------------------------------------------------------------------------
// TST-REG-005: Header enum exact values
// ---------------------------------------------------------------------------

#[test]
fn test_backup_kind_exact_values() {
    assert_eq!(registry::header_enums::BackupKind::Full as u8, 1);
    assert_eq!(registry::header_enums::BackupKind::Differential as u8, 2);
}

#[test]
fn test_platform_hint_exact_values() {
    assert_eq!(registry::header_enums::PlatformHint::Unknown as u8, 0);
    assert_eq!(registry::header_enums::PlatformHint::Windows as u8, 1);
    assert_eq!(registry::header_enums::PlatformHint::Linux as u8, 2);
}

// ---------------------------------------------------------------------------
// TST-REG-006: Registry snapshot test
// ---------------------------------------------------------------------------

#[test]
fn test_registry_snapshot() {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== Record Types ===".into());
    let record_types = [
        registry::record_type::RecordType::Invalid,
        registry::record_type::RecordType::Manifest,
        registry::record_type::RecordType::VolumeSetManifest,
        registry::record_type::RecordType::VolumeFooter,
        registry::record_type::RecordType::Tombstone,
        registry::record_type::RecordType::DataChunk,
        registry::record_type::RecordType::ZeroRun,
        registry::record_type::RecordType::HoleRun,
        registry::record_type::RecordType::FileExtent,
        registry::record_type::RecordType::DiskExtent,
        registry::record_type::RecordType::DiffOperation,
        registry::record_type::RecordType::CatalogPage,
        registry::record_type::RecordType::ChunkIndexPage,
        registry::record_type::RecordType::ErrorRecord,
        registry::record_type::RecordType::FileEntry,
        registry::record_type::RecordType::PartitionLayout,
        registry::record_type::RecordType::BmrArtifact,
        registry::record_type::RecordType::KeySlot,
    ];
    for rt in &record_types {
        lines.push(format!("  {:20} = 0x{:04X}", rt.to_string(), *rt as u16));
    }

    lines.push("=== Feature Bits ===".into());
    let feature_bits: &[(u8, &str)] = &[
        (registry::feature_bit::COMPRESSION_ZSTD, "COMPRESSION_ZSTD"),
        (
            registry::feature_bit::ENCRYPTION_AES256_GCM,
            "ENCRYPTION_AES256_GCM",
        ),
        (registry::feature_bit::VOLUME_SET, "VOLUME_SET"),
        (registry::feature_bit::CHECKPOINT, "CHECKPOINT"),
        (registry::feature_bit::BMR_METADATA, "BMR_METADATA"),
    ];
    for (pos, name) in feature_bits {
        lines.push(format!("  {:32} = bit {}", name, pos));
    }

    lines.push("=== Header Enums ===".into());
    lines.push(format!(
        "  BackupKind::Full         = {}",
        registry::header_enums::BackupKind::Full as u8
    ));
    lines.push(format!(
        "  BackupKind::Differential = {}",
        registry::header_enums::BackupKind::Differential as u8
    ));
    lines.push(format!(
        "  PlatformHint::Unknown    = {}",
        registry::header_enums::PlatformHint::Unknown as u8
    ));
    lines.push(format!(
        "  PlatformHint::Windows    = {}",
        registry::header_enums::PlatformHint::Windows as u8
    ));
    lines.push(format!(
        "  PlatformHint::Linux      = {}",
        registry::header_enums::PlatformHint::Linux as u8
    ));

    let snapshot = lines.join("\n");

    // Build expected programmatically to keep alignment in sync.
    let mut expected = String::new();
    expected.push_str("=== Record Types ===");
    for (name, id) in &[
        ("Invalid", 0x0000u16),
        ("Manifest", 0x0001),
        ("VolumeSetManifest", 0x0002),
        ("VolumeFooter", 0x0003),
        ("Tombstone", 0x0004),
        ("DataChunk", 0x0101),
        ("ZeroRun", 0x0102),
        ("HoleRun", 0x0103),
        ("FileExtent", 0x0104),
        ("DiskExtent", 0x0105),
        ("DiffOperation", 0x0106),
        ("CatalogPage", 0x0201),
        ("ChunkIndexPage", 0x0202),
        ("ErrorRecord", 0x0301),
        ("FileEntry", 0x0401),
        ("PartitionLayout", 0x0402),
        ("BmrArtifact", 0x0403),
        ("KeySlot", 0x0501),
    ] {
        expected.push_str(&format!("\n  {:20} = 0x{:04X}", name, id));
    }
    expected.push_str("\n=== Feature Bits ===");
    for (name, pos) in &[
        ("COMPRESSION_ZSTD", 0u8),
        ("ENCRYPTION_AES256_GCM", 1),
        ("VOLUME_SET", 2),
        ("CHECKPOINT", 3),
        ("BMR_METADATA", 4),
    ] {
        expected.push_str(&format!("\n  {:32} = bit {}", name, pos));
    }
    expected.push_str("\n=== Header Enums ===");
    expected.push_str("\n  BackupKind::Full         = 1");
    expected.push_str("\n  BackupKind::Differential = 2");
    expected.push_str("\n  PlatformHint::Unknown    = 0");
    expected.push_str("\n  PlatformHint::Windows    = 1");
    expected.push_str("\n  PlatformHint::Linux      = 2");

    assert_eq!(snapshot, expected, "Registry snapshot mismatch");
}

// ---------------------------------------------------------------------------
// TST-REG-007: TOML data files correspond to enum variants
// ---------------------------------------------------------------------------

fn parse_toml_entries(text: &str) -> Vec<(&str, Vec<(&str, &str)>)> {
    let mut entries: Vec<(&str, Vec<(&str, &str)>)> = Vec::new();
    let mut current_section: Option<&str> = None;
    let mut current_kvs: Vec<(&str, &str)> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            if let Some(name) = current_section.take() {
                entries.push((name, std::mem::take(&mut current_kvs)));
            }
            let end = trimmed.find(']').expect("malformed TOML section header");
            let raw = &trimmed[1..end];
            let name = raw.rsplit('.').next().unwrap_or(raw);
            current_section = Some(name);
        } else if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let val = trimmed[eq_pos + 1..].trim();
            let val_stripped = val.trim_start_matches('"').trim_end_matches('"');
            current_kvs.push((key, val_stripped));
        }
    }
    if let Some(name) = current_section {
        entries.push((name, current_kvs));
    }
    entries
}

#[test]
fn test_record_types_toml_matches_enum() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let toml_path = manifest_dir.join("registry").join("record_types.toml");
    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("failed to read {toml_path:?}: {e}"));

    let entries = parse_toml_entries(&toml_str);

    let expected_names: HashSet<&str> = [
        "INVALID",
        "MANIFEST",
        "VOLUME_SET_MANIFEST",
        "VOLUME_FOOTER",
        "TOMBSTONE",
        "DATA_CHUNK",
        "ZERO_RUN",
        "HOLE_RUN",
        "FILE_EXTENT",
        "DISK_EXTENT",
        "DIFF_OPERATION",
        "CATALOG_PAGE",
        "CHUNK_INDEX_PAGE",
        "ERROR_RECORD",
        "FILE_ENTRY",
        "PARTITION_LAYOUT",
        "BMR_ARTIFACT",
        "KEY_SLOT",
    ]
    .into();

    let toml_names: HashSet<&str> = entries.iter().map(|(name, _)| *name).collect();

    assert_eq!(
        toml_names.len(),
        18,
        "record_types.toml should have exactly 18 entries, got {}",
        toml_names.len()
    );

    let missing_in_toml: Vec<&&str> = expected_names.difference(&toml_names).collect();
    let extra_in_toml: Vec<&&str> = toml_names.difference(&expected_names).collect();

    assert!(
        missing_in_toml.is_empty(),
        "enum variants missing from record_types.toml: {missing_in_toml:?}"
    );
    assert!(
        extra_in_toml.is_empty(),
        "record_types.toml entries not in enum: {extra_in_toml:?}"
    );

    for (name, kvs) in &entries {
        let id_str = kvs
            .iter()
            .find(|(k, _)| *k == "id")
            .map(|(_, v)| *v)
            .unwrap_or("");
        let expected_id: u16 = match *name {
            "INVALID" => 0x0000,
            "MANIFEST" => 0x0001,
            "VOLUME_SET_MANIFEST" => 0x0002,
            "VOLUME_FOOTER" => 0x0003,
            "TOMBSTONE" => 0x0004,
            "DATA_CHUNK" => 0x0101,
            "ZERO_RUN" => 0x0102,
            "HOLE_RUN" => 0x0103,
            "FILE_EXTENT" => 0x0104,
            "DISK_EXTENT" => 0x0105,
            "DIFF_OPERATION" => 0x0106,
            "CATALOG_PAGE" => 0x0201,
            "CHUNK_INDEX_PAGE" => 0x0202,
            "ERROR_RECORD" => 0x0301,
            "FILE_ENTRY" => 0x0401,
            "PARTITION_LAYOUT" => 0x0402,
            "BMR_ARTIFACT" => 0x0403,
            "KEY_SLOT" => 0x0501,
            other => panic!("unexpected record type in TOML: {other}"),
        };
        assert_eq!(
            id_str,
            &format!("0x{expected_id:04X}"),
            "record_types.toml {name}: expected id 0x{expected_id:04X}, got {id_str}"
        );
    }
}

#[test]
fn test_feature_bits_toml_matches_constants() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let toml_path = manifest_dir.join("registry").join("feature_bits.toml");
    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("failed to read {toml_path:?}: {e}"));

    let entries = parse_toml_entries(&toml_str);

    let expected: &[(&str, u8)] = &[
        ("COMPRESSION_ZSTD", registry::feature_bit::COMPRESSION_ZSTD),
        (
            "ENCRYPTION_AES256_GCM",
            registry::feature_bit::ENCRYPTION_AES256_GCM,
        ),
        ("VOLUME_SET", registry::feature_bit::VOLUME_SET),
        ("CHECKPOINT", registry::feature_bit::CHECKPOINT),
        ("BMR_METADATA", registry::feature_bit::BMR_METADATA),
    ];

    assert_eq!(
        entries.len(),
        expected.len(),
        "expected {} feature bit entries",
        expected.len()
    );

    for (name, expected_bit) in expected {
        let entry = entries
            .iter()
            .find(|(n, _)| *n == *name)
            .unwrap_or_else(|| panic!("feature_bits.toml missing entry for {name}"));

        let bit_str = entry
            .1
            .iter()
            .find(|(k, _)| *k == "bit")
            .map(|(_, v)| *v)
            .unwrap_or("");
        let parsed_bit: u8 = bit_str.parse().unwrap_or_else(|e| {
            panic!("feature_bits.toml {name}: invalid bit value {bit_str:?}: {e}")
        });

        assert_eq!(
            parsed_bit, *expected_bit,
            "feature_bits.toml {name}: expected bit {}, got {}",
            expected_bit, parsed_bit,
        );
    }
}

// ---------------------------------------------------------------------------
// TST-REG-005 extended: BackupKind / PlatformHint Display counts
// ---------------------------------------------------------------------------

#[test]
fn test_backup_kind_display_and_count() {
    use registry::header_enums::BackupKind;

    let cases: &[(BackupKind, u8, &str)] = &[
        (BackupKind::Full, 1, "Full"),
        (BackupKind::Differential, 2, "Differential"),
    ];

    assert_eq!(cases.len(), 2, "expected exactly 2 BackupKind variants");

    for (variant, expected_value, expected_name) in cases {
        assert_eq!(
            *variant as u8, *expected_value,
            "BackupKind discriminant mismatch for {expected_name}"
        );
        assert_eq!(
            variant.to_string(),
            *expected_name,
            "BackupKind Display mismatch for {expected_name}"
        );
    }
}

#[test]
fn test_platform_hint_display_and_count() {
    use registry::header_enums::PlatformHint;

    let cases: &[(PlatformHint, u8, &str)] = &[
        (PlatformHint::Unknown, 0, "Unknown"),
        (PlatformHint::Windows, 1, "Windows"),
        (PlatformHint::Linux, 2, "Linux"),
    ];

    assert_eq!(cases.len(), 3, "expected exactly 3 PlatformHint variants");

    for (variant, expected_value, expected_name) in cases {
        assert_eq!(
            *variant as u8, *expected_value,
            "PlatformHint discriminant mismatch for {expected_name}"
        );
        assert_eq!(
            variant.to_string(),
            *expected_name,
            "PlatformHint Display mismatch for {expected_name}"
        );
    }
}

// ---------------------------------------------------------------------------
// TST-REG-00x: ErrorId TOML consistency
// ---------------------------------------------------------------------------

#[test]
fn test_error_id_all_variants_matchable() {
    let cases: &[(registry::error_id::ErrorId, u16, &str)] = &[
        (registry::error_id::ErrorId::Invalid, 0x0000, "Invalid"),
        (
            registry::error_id::ErrorId::ChecksumMismatch,
            0x0001,
            "ChecksumMismatch",
        ),
        (
            registry::error_id::ErrorId::FormatVersion,
            0x0002,
            "FormatVersion",
        ),
        (
            registry::error_id::ErrorId::Compression,
            0x0003,
            "Compression",
        ),
        (
            registry::error_id::ErrorId::SegmentCorrupt,
            0x0004,
            "SegmentCorrupt",
        ),
        (registry::error_id::ErrorId::IoError, 0x0100, "IoError"),
        (
            registry::error_id::ErrorId::VolumeMissing,
            0x0101,
            "VolumeMissing",
        ),
        (
            registry::error_id::ErrorId::EncryptionAuth,
            0x0200,
            "EncryptionAuth",
        ),
        (
            registry::error_id::ErrorId::CatalogCorrupt,
            0x0300,
            "CatalogCorrupt",
        ),
    ];

    assert_eq!(cases.len(), 9, "expected exactly 9 ErrorId variants");

    for (variant, expected_id, expected_name) in cases {
        assert_eq!(
            *variant as u16, *expected_id,
            "ErrorId discriminant mismatch for {expected_name}"
        );
        assert_eq!(
            variant.to_string(),
            *expected_name,
            "ErrorId Display mismatch for {expected_name}"
        );
    }
}

#[test]
fn test_error_ids_toml_matches_enum() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let toml_path = manifest_dir.join("registry").join("error_ids.toml");
    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("failed to read {toml_path:?}: {e}"));

    let entries = parse_toml_entries(&toml_str);

    let expected_names: HashSet<&str> = [
        "INVALID",
        "CHECKSUM_MISMATCH",
        "FORMAT_VERSION",
        "COMPRESSION",
        "SEGMENT_CORRUPT",
        "IO_ERROR",
        "VOLUME_MISSING",
        "ENCRYPTION_AUTH",
        "CATALOG_CORRUPT",
    ]
    .into();

    let toml_names: HashSet<&str> = entries.iter().map(|(name, _)| *name).collect();

    assert_eq!(
        toml_names.len(),
        9,
        "error_ids.toml should have exactly 9 entries, got {}",
        toml_names.len()
    );

    let missing_in_toml: Vec<&&str> = expected_names.difference(&toml_names).collect();
    let extra_in_toml: Vec<&&str> = toml_names.difference(&expected_names).collect();

    assert!(
        missing_in_toml.is_empty(),
        "enum variants missing from error_ids.toml: {missing_in_toml:?}"
    );
    assert!(
        extra_in_toml.is_empty(),
        "error_ids.toml entries not in enum: {extra_in_toml:?}"
    );

    for (name, kvs) in &entries {
        let id_str = kvs
            .iter()
            .find(|(k, _)| *k == "id")
            .map(|(_, v)| *v)
            .unwrap_or("");
        let expected_id: u16 = match *name {
            "INVALID" => 0x0000,
            "CHECKSUM_MISMATCH" => 0x0001,
            "FORMAT_VERSION" => 0x0002,
            "COMPRESSION" => 0x0003,
            "SEGMENT_CORRUPT" => 0x0004,
            "IO_ERROR" => 0x0100,
            "VOLUME_MISSING" => 0x0101,
            "ENCRYPTION_AUTH" => 0x0200,
            "CATALOG_CORRUPT" => 0x0300,
            other => panic!("unexpected error id in TOML: {other}"),
        };
        assert_eq!(
            id_str,
            &format!("0x{expected_id:04X}"),
            "error_ids.toml {name}: expected id 0x{expected_id:04X}, got {id_str}"
        );
    }
}
