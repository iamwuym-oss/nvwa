//! Integration tests for the NWB Format Registry.
//!
//! Coverage:
//!
//! | ID        | Target                                          |
//! |-----------|-------------------------------------------------|
//! | TST-REG-001 | All 18 RecordType variants exist and are matchable |
//! | TST-REG-002 | Duplicate ErrorId discriminant rejected by compile-fail test |
//! | TST-REG-003 | 5 Feature bits — no two share the same position  |
//! | TST-REG-004 | All bit positions within 0..63                  |
//! | TST-REG-005 | BackupKind / PlatformHint have exact values      |
//! | TST-REG-006 | Registry snapshot matches expected output         |
//! | TST-REG-007 | Generated Rust matches committed files byte-for-byte |

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
// TST-REG-002 is implemented by tests/compile_fail.rs using trybuild.
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
    use std::mem::{size_of, size_of_val};

    use registry::error_id::ErrorId;
    use registry::header_enums::{BackupKind, PlatformHint};
    use registry::record_type::RecordType;

    assert_eq!(size_of::<RecordType>(), size_of::<u16>());
    assert_eq!(size_of::<ErrorId>(), size_of::<u16>());
    assert_eq!(size_of::<BackupKind>(), size_of::<u8>());
    assert_eq!(size_of::<PlatformHint>(), size_of::<u8>());
    assert_eq!(
        size_of_val(&registry::feature_bit::COMPRESSION_ZSTD),
        size_of::<u8>(),
    );

    let mut lines: Vec<String> = Vec::new();

    lines.push("=== RecordType (u16) ===".into());
    let record_types: &[(&str, RecordType)] = &[
        ("Invalid", RecordType::Invalid),
        ("Manifest", RecordType::Manifest),
        ("VolumeSetManifest", RecordType::VolumeSetManifest),
        ("VolumeFooter", RecordType::VolumeFooter),
        ("Tombstone", RecordType::Tombstone),
        ("DataChunk", RecordType::DataChunk),
        ("ZeroRun", RecordType::ZeroRun),
        ("HoleRun", RecordType::HoleRun),
        ("FileExtent", RecordType::FileExtent),
        ("DiskExtent", RecordType::DiskExtent),
        ("DiffOperation", RecordType::DiffOperation),
        ("CatalogPage", RecordType::CatalogPage),
        ("ChunkIndexPage", RecordType::ChunkIndexPage),
        ("ErrorRecord", RecordType::ErrorRecord),
        ("FileEntry", RecordType::FileEntry),
        ("PartitionLayout", RecordType::PartitionLayout),
        ("BmrArtifact", RecordType::BmrArtifact),
        ("KeySlot", RecordType::KeySlot),
    ];
    for &(name, variant) in record_types {
        lines.push(format!(
            "RecordType::{name}|value=0x{:04X}|display={variant}",
            variant as u16
        ));
    }

    lines.push("=== FeatureBit (u8) ===".into());
    let feature_bits: &[(&str, u8)] = &[
        ("COMPRESSION_ZSTD", registry::feature_bit::COMPRESSION_ZSTD),
        (
            "ENCRYPTION_AES256_GCM",
            registry::feature_bit::ENCRYPTION_AES256_GCM,
        ),
        ("VOLUME_SET", registry::feature_bit::VOLUME_SET),
        ("CHECKPOINT", registry::feature_bit::CHECKPOINT),
        ("BMR_METADATA", registry::feature_bit::BMR_METADATA),
    ];
    for &(name, bit) in feature_bits {
        lines.push(format!("FeatureBit::{name}|bit={bit}"));
    }

    lines.push("=== BackupKind (u8) ===".into());
    let backup_kinds: &[(&str, BackupKind)] = &[
        ("Full", BackupKind::Full),
        ("Differential", BackupKind::Differential),
    ];
    for &(name, variant) in backup_kinds {
        lines.push(format!(
            "BackupKind::{name}|value={}|display={variant}",
            variant as u8
        ));
    }

    lines.push("=== PlatformHint (u8) ===".into());
    let platform_hints: &[(&str, PlatformHint)] = &[
        ("Unknown", PlatformHint::Unknown),
        ("Windows", PlatformHint::Windows),
        ("Linux", PlatformHint::Linux),
    ];
    for &(name, variant) in platform_hints {
        lines.push(format!(
            "PlatformHint::{name}|value={}|display={variant}",
            variant as u8
        ));
    }

    lines.push("=== ErrorId (u16) ===".into());
    let error_ids: &[(&str, ErrorId)] = &[
        ("Invalid", ErrorId::Invalid),
        ("ChecksumMismatch", ErrorId::ChecksumMismatch),
        ("FormatVersion", ErrorId::FormatVersion),
        ("Compression", ErrorId::Compression),
        ("SegmentCorrupt", ErrorId::SegmentCorrupt),
        ("IoError", ErrorId::IoError),
        ("VolumeMissing", ErrorId::VolumeMissing),
        ("EncryptionAuth", ErrorId::EncryptionAuth),
        ("CatalogCorrupt", ErrorId::CatalogCorrupt),
    ];
    for &(name, variant) in error_ids {
        lines.push(format!(
            "ErrorId::{name}|value=0x{:04X}|display={variant}",
            variant as u16
        ));
    }

    let snapshot = lines.join("\n");
    assert_eq!(
        snapshot,
        r#"=== RecordType (u16) ===
RecordType::Invalid|value=0x0000|display=Invalid
RecordType::Manifest|value=0x0001|display=Manifest
RecordType::VolumeSetManifest|value=0x0002|display=VolumeSetManifest
RecordType::VolumeFooter|value=0x0003|display=VolumeFooter
RecordType::Tombstone|value=0x0004|display=Tombstone
RecordType::DataChunk|value=0x0101|display=DataChunk
RecordType::ZeroRun|value=0x0102|display=ZeroRun
RecordType::HoleRun|value=0x0103|display=HoleRun
RecordType::FileExtent|value=0x0104|display=FileExtent
RecordType::DiskExtent|value=0x0105|display=DiskExtent
RecordType::DiffOperation|value=0x0106|display=DiffOperation
RecordType::CatalogPage|value=0x0201|display=CatalogPage
RecordType::ChunkIndexPage|value=0x0202|display=ChunkIndexPage
RecordType::ErrorRecord|value=0x0301|display=ErrorRecord
RecordType::FileEntry|value=0x0401|display=FileEntry
RecordType::PartitionLayout|value=0x0402|display=PartitionLayout
RecordType::BmrArtifact|value=0x0403|display=BmrArtifact
RecordType::KeySlot|value=0x0501|display=KeySlot
=== FeatureBit (u8) ===
FeatureBit::COMPRESSION_ZSTD|bit=0
FeatureBit::ENCRYPTION_AES256_GCM|bit=1
FeatureBit::VOLUME_SET|bit=2
FeatureBit::CHECKPOINT|bit=3
FeatureBit::BMR_METADATA|bit=4
=== BackupKind (u8) ===
BackupKind::Full|value=1|display=Full
BackupKind::Differential|value=2|display=Differential
=== PlatformHint (u8) ===
PlatformHint::Unknown|value=0|display=Unknown
PlatformHint::Windows|value=1|display=Windows
PlatformHint::Linux|value=2|display=Linux
=== ErrorId (u16) ===
ErrorId::Invalid|value=0x0000|display=Invalid
ErrorId::ChecksumMismatch|value=0x0001|display=ChecksumMismatch
ErrorId::FormatVersion|value=0x0002|display=FormatVersion
ErrorId::Compression|value=0x0003|display=Compression
ErrorId::SegmentCorrupt|value=0x0004|display=SegmentCorrupt
ErrorId::IoError|value=0x0100|display=IoError
ErrorId::VolumeMissing|value=0x0101|display=VolumeMissing
ErrorId::EncryptionAuth|value=0x0200|display=EncryptionAuth
ErrorId::CatalogCorrupt|value=0x0300|display=CatalogCorrupt"#,
    );
}

#[test]
fn test_authoritative_toml_registry_snapshot() {
    use format_registry_generator::parser::{parse_and_validate, ParsedRegistry};

    let mut lines: Vec<String> = Vec::new();

    let record_types = parse_and_validate(include_str!("../registry/record_types.toml"))
        .expect("record_types.toml must parse and pass semantic validation");
    assert_eq!(record_types.kind(), "discriminant");
    match record_types {
        ParsedRegistry::Discriminant(mut registry) => {
            assert_eq!(registry.enum_name, "RecordType");
            assert_eq!(registry.entries.len(), 18);
            registry.entries.sort_by_key(|entry| entry.value);
            lines.push(format!(
                "registry=record_types|kind=discriminant|enum_name={}|entries={}",
                registry.enum_name,
                registry.entries.len()
            ));
            for entry in registry.entries {
                lines.push(format!(
                    "RecordType|canonical={}|rust={}|value=0x{:04X}",
                    entry.canonical_name, entry.rust_name, entry.value
                ));
            }
        }
        other => panic!("record_types.toml parsed as unexpected kind: {other:?}"),
    }

    let feature_bits = parse_and_validate(include_str!("../registry/feature_bits.toml"))
        .expect("feature_bits.toml must parse and pass semantic validation");
    assert_eq!(feature_bits.kind(), "bit");
    match feature_bits {
        ParsedRegistry::Bit(mut entries) => {
            assert_eq!(entries.len(), 5);
            entries.sort_by_key(|entry| entry.bit);
            lines.push(format!(
                "registry=feature_bits|kind=bit|entries={}",
                entries.len()
            ));
            for entry in entries {
                lines.push(format!(
                    "FeatureBit|canonical={}|rust={}|bit={}",
                    entry.canonical_name, entry.rust_name, entry.bit
                ));
            }
        }
        other => panic!("feature_bits.toml parsed as unexpected kind: {other:?}"),
    }

    let header_enums = parse_and_validate(include_str!("../registry/header_enums.toml"))
        .expect("header_enums.toml must parse and pass semantic validation");
    assert_eq!(header_enums.kind(), "enum_set");
    match header_enums {
        ParsedRegistry::EnumSet(mut enums) => {
            assert_eq!(enums.len(), 2);
            assert_eq!(
                enums
                    .iter()
                    .map(|definition| definition.entries.len())
                    .sum::<usize>(),
                5
            );
            enums.sort_by(|left, right| left.canonical_name.cmp(&right.canonical_name));
            lines.push(format!(
                "registry=header_enums|kind=enum_set|enums={}|entries={}",
                enums.len(),
                enums
                    .iter()
                    .map(|definition| definition.entries.len())
                    .sum::<usize>()
            ));
            for mut definition in enums {
                assert_eq!(definition.repr, "u8");
                definition.entries.sort_by_key(|entry| entry.value);
                lines.push(format!(
                    "HeaderEnum|canonical={}|rust={}|repr={}|entries={}",
                    definition.canonical_name,
                    definition.rust_name,
                    definition.repr,
                    definition.entries.len()
                ));
                for entry in definition.entries {
                    lines.push(format!(
                        "HeaderEnumEntry|enum={}|canonical={}|rust={}|value={}",
                        definition.rust_name, entry.canonical_name, entry.rust_name, entry.value
                    ));
                }
            }
        }
        other => panic!("header_enums.toml parsed as unexpected kind: {other:?}"),
    }

    let error_ids = parse_and_validate(include_str!("../registry/error_ids.toml"))
        .expect("error_ids.toml must parse and pass semantic validation");
    assert_eq!(error_ids.kind(), "discriminant");
    match error_ids {
        ParsedRegistry::Discriminant(mut registry) => {
            assert_eq!(registry.enum_name, "ErrorId");
            assert_eq!(registry.entries.len(), 9);
            registry.entries.sort_by_key(|entry| entry.value);
            lines.push(format!(
                "registry=error_ids|kind=discriminant|enum_name={}|entries={}",
                registry.enum_name,
                registry.entries.len()
            ));
            for entry in registry.entries {
                lines.push(format!(
                    "ErrorId|canonical={}|rust={}|value=0x{:04X}",
                    entry.canonical_name, entry.rust_name, entry.value
                ));
            }
        }
        other => panic!("error_ids.toml parsed as unexpected kind: {other:?}"),
    }

    assert_eq!(
        lines.join("\n"),
        r#"registry=record_types|kind=discriminant|enum_name=RecordType|entries=18
RecordType|canonical=INVALID|rust=Invalid|value=0x0000
RecordType|canonical=MANIFEST|rust=Manifest|value=0x0001
RecordType|canonical=VOLUME_SET_MANIFEST|rust=VolumeSetManifest|value=0x0002
RecordType|canonical=VOLUME_FOOTER|rust=VolumeFooter|value=0x0003
RecordType|canonical=TOMBSTONE|rust=Tombstone|value=0x0004
RecordType|canonical=DATA_CHUNK|rust=DataChunk|value=0x0101
RecordType|canonical=ZERO_RUN|rust=ZeroRun|value=0x0102
RecordType|canonical=HOLE_RUN|rust=HoleRun|value=0x0103
RecordType|canonical=FILE_EXTENT|rust=FileExtent|value=0x0104
RecordType|canonical=DISK_EXTENT|rust=DiskExtent|value=0x0105
RecordType|canonical=DIFF_OPERATION|rust=DiffOperation|value=0x0106
RecordType|canonical=CATALOG_PAGE|rust=CatalogPage|value=0x0201
RecordType|canonical=CHUNK_INDEX_PAGE|rust=ChunkIndexPage|value=0x0202
RecordType|canonical=ERROR_RECORD|rust=ErrorRecord|value=0x0301
RecordType|canonical=FILE_ENTRY|rust=FileEntry|value=0x0401
RecordType|canonical=PARTITION_LAYOUT|rust=PartitionLayout|value=0x0402
RecordType|canonical=BMR_ARTIFACT|rust=BmrArtifact|value=0x0403
RecordType|canonical=KEY_SLOT|rust=KeySlot|value=0x0501
registry=feature_bits|kind=bit|entries=5
FeatureBit|canonical=COMPRESSION_ZSTD|rust=COMPRESSION_ZSTD|bit=0
FeatureBit|canonical=ENCRYPTION_AES256_GCM|rust=ENCRYPTION_AES256_GCM|bit=1
FeatureBit|canonical=VOLUME_SET|rust=VOLUME_SET|bit=2
FeatureBit|canonical=CHECKPOINT|rust=CHECKPOINT|bit=3
FeatureBit|canonical=BMR_METADATA|rust=BMR_METADATA|bit=4
registry=header_enums|kind=enum_set|enums=2|entries=5
HeaderEnum|canonical=BACKUP_KIND|rust=BackupKind|repr=u8|entries=2
HeaderEnumEntry|enum=BackupKind|canonical=FULL|rust=Full|value=1
HeaderEnumEntry|enum=BackupKind|canonical=DIFFERENTIAL|rust=Differential|value=2
HeaderEnum|canonical=PLATFORM_HINT|rust=PlatformHint|repr=u8|entries=3
HeaderEnumEntry|enum=PlatformHint|canonical=UNKNOWN|rust=Unknown|value=0
HeaderEnumEntry|enum=PlatformHint|canonical=WINDOWS|rust=Windows|value=1
HeaderEnumEntry|enum=PlatformHint|canonical=LINUX|rust=Linux|value=2
registry=error_ids|kind=discriminant|enum_name=ErrorId|entries=9
ErrorId|canonical=INVALID|rust=Invalid|value=0x0000
ErrorId|canonical=CHECKSUM_MISMATCH|rust=ChecksumMismatch|value=0x0001
ErrorId|canonical=FORMAT_VERSION|rust=FormatVersion|value=0x0002
ErrorId|canonical=COMPRESSION|rust=Compression|value=0x0003
ErrorId|canonical=SEGMENT_CORRUPT|rust=SegmentCorrupt|value=0x0004
ErrorId|canonical=IO_ERROR|rust=IoError|value=0x0100
ErrorId|canonical=VOLUME_MISSING|rust=VolumeMissing|value=0x0101
ErrorId|canonical=ENCRYPTION_AUTH|rust=EncryptionAuth|value=0x0200
ErrorId|canonical=CATALOG_CORRUPT|rust=CatalogCorrupt|value=0x0300"#,
    );
}

#[test]
fn test_generated_registry_repr_and_constant_types() {
    let record_type_source = include_str!("../src/registry/record_type.rs");
    let error_id_source = include_str!("../src/registry/error_id.rs");
    let header_enums_source = include_str!("../src/registry/header_enums.rs");
    let feature_bit_source = include_str!("../src/registry/feature_bit.rs");

    let record_repr: Vec<&str> = record_type_source
        .lines()
        .filter(|line| line.starts_with("#[repr("))
        .collect();
    assert_eq!(record_repr.as_slice(), &["#[repr(u16)]"]);
    assert_eq!(
        record_type_source.matches("pub enum RecordType {").count(),
        1
    );
    assert_eq!(
        record_type_source
            .matches("#[repr(u16)]\n#[allow(clippy::upper_case_acronyms)]\npub enum RecordType {",)
            .count(),
        1
    );

    let error_repr: Vec<&str> = error_id_source
        .lines()
        .filter(|line| line.starts_with("#[repr("))
        .collect();
    assert_eq!(error_repr.as_slice(), &["#[repr(u16)]"]);
    assert_eq!(error_id_source.matches("pub enum ErrorId {").count(), 1);
    assert_eq!(
        error_id_source
            .matches("#[repr(u16)]\n#[allow(clippy::upper_case_acronyms)]\npub enum ErrorId {")
            .count(),
        1
    );

    let header_repr: Vec<&str> = header_enums_source
        .lines()
        .filter(|line| line.starts_with("#[repr("))
        .collect();
    assert_eq!(header_repr.as_slice(), &["#[repr(u8)]", "#[repr(u8)]"],);
    assert_eq!(
        header_enums_source
            .matches("#[repr(u8)]\npub enum BackupKind {")
            .count(),
        1
    );
    assert_eq!(
        header_enums_source
            .matches("#[repr(u8)]\npub enum PlatformHint {")
            .count(),
        1
    );

    let feature_constants: Vec<&str> = feature_bit_source
        .lines()
        .filter(|line| line.starts_with("pub const "))
        .collect();
    assert_eq!(
        feature_constants.as_slice(),
        &[
            "pub const BMR_METADATA: u8 = 4;",
            "pub const CHECKPOINT: u8 = 3;",
            "pub const COMPRESSION_ZSTD: u8 = 0;",
            "pub const ENCRYPTION_AES256_GCM: u8 = 1;",
            "pub const VOLUME_SET: u8 = 2;",
        ],
    );
}

// ---------------------------------------------------------------------------
// TST-REG-007: Generated Rust matches committed files byte-for-byte
//
// Uses the format-registry-generator library's check function to verify that
// the generated .rs files on disk are in sync with the schema v1 TOML sources.
// ---------------------------------------------------------------------------

#[test]
fn test_generated_files_match_toml_sources() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    format_registry_generator::check_registry(manifest_dir)
        .expect("generated .rs files are stale or TOML sources are invalid");
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
// ErrorId exact value tests
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
