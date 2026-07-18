//! Negative validation tests for the Format Registry generator.
//!
//! Coverage targets (reviewer finding IMP-001-GENERATOR-REMEDIATION-1):
//!
//!  1. Duplicate discriminant values in validate_discriminant_entries()
//!  2. Duplicate canonical names in validate_discriminant_entries()
//!  3. Duplicate rust names in validate_discriminant_entries()
//!  4. Reserved 0x0000 conflict in validate_discriminant_entries()
//!  5. Duplicate bit positions in validate_bit_entries()
//!  6. Bit positions > 63 in validate_bit_entries()
//!  7. Duplicate canonical names in validate_bit_entries()
//!  8. Duplicate rust names in validate_bit_entries()
//!  9. Duplicate enum values in validate_enum_def()
//! 10. Duplicate canonical names in validate_enum_def()
//! 11. Duplicate rust names in validate_enum_def()
//! 12. parse_discriminant: missing required fields (rust_name)
//! 13. parse_discriminant: missing required fields (value)
//! 14. parse_discriminant: invalid hex string
//! 15. parse_discriminant: duplicate values via parse_and_validate
//! 16. parse_bit: missing required fields (rust_name)
//! 17. parse_bit: missing required fields (bit)
//! 18. parse_bit: bit value out of u8 range
//! 19. parse_bit: bit position > 63 via parse_and_validate
//! 20. parse_enum_set: invalid repr
//! 21. parse_enum_set: empty enums section
//! 22. parse_enum_set: missing entries section
//! 23. parse_enum_set: entry value out of u8 range
//! 24. parse_enum_set: missing required fields (rust_name)
//! 25. parse_enum_set: missing required fields (value)
//! 26. parse_discriminant: hex value out of u16 range

use format_registry_generator::parser;
use format_registry_generator::schema::{
    validate_bit_entries, validate_discriminant_entries, validate_enum_def, BitEntry,
    DiscriminantEntry, EnumDef, EnumVariant, ValidationError,
};

// =========================================================================
// Direct validator tests (schema.rs)
// =========================================================================

// --- (1) Duplicate discriminant values ----------------------------------

#[test]
fn test_duplicate_discriminant_values() {
    let entries = vec![
        DiscriminantEntry {
            canonical_name: "ALPHA".into(),
            rust_name: "Alpha".into(),
            value: 0x0001,
            description: "first".into(),
        },
        DiscriminantEntry {
            canonical_name: "BETA".into(),
            rust_name: "Beta".into(),
            value: 0x0001,
            description: "duplicate value".into(),
        },
    ];
    let err = validate_discriminant_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateValue(..))),
        "expected DuplicateValue error, got: {:?}",
        err
    );
}

// --- (2) Duplicate canonical names in discriminants ---------------------

#[test]
fn test_duplicate_discriminant_canonical_name() {
    let entries = vec![
        DiscriminantEntry {
            canonical_name: "DUPE".into(),
            rust_name: "One".into(),
            value: 0x0001,
            description: "".into(),
        },
        DiscriminantEntry {
            canonical_name: "DUPE".into(),
            rust_name: "Two".into(),
            value: 0x0002,
            description: "".into(),
        },
    ];
    let err = validate_discriminant_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateCanonicalName(..))),
        "expected DuplicateCanonicalName error, got: {:?}",
        err
    );
}

// --- (3) Duplicate rust names in discriminants --------------------------

#[test]
fn test_duplicate_discriminant_rust_name() {
    let entries = vec![
        DiscriminantEntry {
            canonical_name: "FIRST".into(),
            rust_name: "Same".into(),
            value: 0x0001,
            description: "".into(),
        },
        DiscriminantEntry {
            canonical_name: "SECOND".into(),
            rust_name: "Same".into(),
            value: 0x0002,
            description: "".into(),
        },
    ];
    let err = validate_discriminant_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateRustName(..))),
        "expected DuplicateRustName error, got: {:?}",
        err
    );
}

// --- (4) Reserved 0x0000 conflict ---------------------------------------

#[test]
fn test_discriminant_reserved_0000_non_invalid() {
    let entries = vec![DiscriminantEntry {
        canonical_name: "SOMETHING".into(),
        rust_name: "Something".into(),
        value: 0x0000,
        description: "should be reserved".into(),
    }];
    let err = validate_discriminant_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::ReservedConflict(..))),
        "expected ReservedConflict error, got: {:?}",
        err
    );
}

// --- (5) Duplicate bit positions ----------------------------------------

#[test]
fn test_bit_duplicate_position() {
    let entries = vec![
        BitEntry {
            canonical_name: "BIT_A".into(),
            rust_name: "BitA".into(),
            bit: 5,
            description: "".into(),
        },
        BitEntry {
            canonical_name: "BIT_B".into(),
            rust_name: "BitB".into(),
            bit: 5,
            description: "".into(),
        },
    ];
    let err = validate_bit_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateValue(..))),
        "expected DuplicateValue error, got: {:?}",
        err
    );
}

// --- (6) Bit position > 63 ----------------------------------------------

#[test]
fn test_bit_position_exceeds_63() {
    let entries = vec![BitEntry {
        canonical_name: "BAD_BIT".into(),
        rust_name: "BadBit".into(),
        bit: 64,
        description: "".into(),
    }];
    let err = validate_bit_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::BitOutOfRange(..))),
        "expected BitOutOfRange error, got: {:?}",
        err
    );
}

// --- (7) Duplicate canonical names in bits ------------------------------

#[test]
fn test_bit_duplicate_canonical_name() {
    let entries = vec![
        BitEntry {
            canonical_name: "DUPE".into(),
            rust_name: "One".into(),
            bit: 1,
            description: "".into(),
        },
        BitEntry {
            canonical_name: "DUPE".into(),
            rust_name: "Two".into(),
            bit: 2,
            description: "".into(),
        },
    ];
    let err = validate_bit_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateCanonicalName(..))),
        "expected DuplicateCanonicalName error, got: {:?}",
        err
    );
}

// --- (8) Duplicate rust names in bits -----------------------------------

#[test]
fn test_bit_duplicate_rust_name() {
    let entries = vec![
        BitEntry {
            canonical_name: "FIRST".into(),
            rust_name: "Same".into(),
            bit: 1,
            description: "".into(),
        },
        BitEntry {
            canonical_name: "SECOND".into(),
            rust_name: "Same".into(),
            bit: 2,
            description: "".into(),
        },
    ];
    let err = validate_bit_entries(&entries).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateRustName(..))),
        "expected DuplicateRustName error, got: {:?}",
        err
    );
}

// --- (9) Duplicate enum values ------------------------------------------

#[test]
fn test_enum_duplicate_value() {
    let def = EnumDef {
        canonical_name: "TestEnum".into(),
        rust_name: "TestEnum".into(),
        repr: "u8".into(),
        entries: vec![
            EnumVariant {
                canonical_name: "VAR_A".into(),
                rust_name: "VarA".into(),
                value: 1,
                description: "".into(),
            },
            EnumVariant {
                canonical_name: "VAR_B".into(),
                rust_name: "VarB".into(),
                value: 1,
                description: "".into(),
            },
        ],
    };
    let err = validate_enum_def(&def).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateEnumValue(..))),
        "expected DuplicateEnumValue error, got: {:?}",
        err
    );
}

// --- (10) Duplicate canonical names in enum variants --------------------

#[test]
fn test_enum_duplicate_canonical_name() {
    let def = EnumDef {
        canonical_name: "TestEnum".into(),
        rust_name: "TestEnum".into(),
        repr: "u8".into(),
        entries: vec![
            EnumVariant {
                canonical_name: "DUPE".into(),
                rust_name: "One".into(),
                value: 1,
                description: "".into(),
            },
            EnumVariant {
                canonical_name: "DUPE".into(),
                rust_name: "Two".into(),
                value: 2,
                description: "".into(),
            },
        ],
    };
    let err = validate_enum_def(&def).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateCanonicalName(..))),
        "expected DuplicateCanonicalName error, got: {:?}",
        err
    );
}

// --- (11) Duplicate rust names in enum variants -------------------------

#[test]
fn test_enum_duplicate_rust_name() {
    let def = EnumDef {
        canonical_name: "TestEnum".into(),
        rust_name: "TestEnum".into(),
        repr: "u8".into(),
        entries: vec![
            EnumVariant {
                canonical_name: "FIRST".into(),
                rust_name: "Same".into(),
                value: 1,
                description: "".into(),
            },
            EnumVariant {
                canonical_name: "SECOND".into(),
                rust_name: "Same".into(),
                value: 2,
                description: "".into(),
            },
        ],
    };
    let err = validate_enum_def(&def).unwrap_err();
    assert!(
        err.iter()
            .any(|e| matches!(e, ValidationError::DuplicateRustName(..))),
        "expected DuplicateRustName error, got: {:?}",
        err
    );
}

// =========================================================================
// Parser-level negative tests (parser.rs via parse_and_validate)
// =========================================================================

// --- (12) parse_discriminant: missing rust_name -------------------------

#[test]
fn test_parse_discriminant_missing_rust_name() {
    let toml = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
value = 0x0001
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("rust_name") || msg.contains("missing"),
        "expected error about missing rust_name, got: {msg}"
    );
}

// --- (13) parse_discriminant: missing value -----------------------------

#[test]
fn test_parse_discriminant_missing_value() {
    let toml = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("value") || msg.contains("missing"),
        "expected error about missing value, got: {msg}"
    );
}

// --- (14) parse_discriminant: invalid hex string ------------------------

#[test]
fn test_parse_discriminant_invalid_hex_string() {
    let toml = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
value = "0xZZZZ"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("invalid hex") || msg.contains("error"),
        "expected error about invalid hex, got: {msg}"
    );
}

// --- (15) parse_discriminant: duplicate values via toml -----------------

#[test]
fn test_parse_discriminant_toml_duplicate_values() {
    let toml = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
value = 0x0001

[entries.BAR]
rust_name = "Bar"
value = 0x0001
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("duplicate value") || msg.contains("DuplicateValue") || msg.contains("0x0001"),
        "expected error about duplicate value, got: {msg}"
    );
}

// --- (16) parse_bit: missing rust_name ----------------------------------

#[test]
fn test_parse_bit_missing_rust_name() {
    let toml = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.TEST_BIT]
bit = 1
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("rust_name") || msg.contains("missing"),
        "expected error about missing rust_name, got: {msg}"
    );
}

// --- (17) parse_bit: missing bit field ----------------------------------

#[test]
fn test_parse_bit_missing_bit_field() {
    let toml = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.TEST_BIT]
rust_name = "TestBit"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("bit") || msg.contains("missing"),
        "expected error about missing bit field, got: {msg}"
    );
}

// --- (18) parse_bit: bit value out of u8 range --------------------------

#[test]
fn test_parse_bit_value_out_of_u8_range() {
    let toml = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.TEST_BIT]
rust_name = "TestBit"
bit = 256
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("out of u8 range") || msg.contains("range"),
        "expected error about bit out of range, got: {msg}"
    );
}

// --- (19) parse_bit: bit position > 63 via parse_and_validate -----------

#[test]
fn test_parse_bit_position_exceeds_63_via_parser() {
    let toml = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.BAD_BIT]
rust_name = "BadBit"
bit = 64
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("out of range") || msg.contains("BitOutOfRange"),
        "expected error about bit > 63, got: {msg}"
    );
}

// --- (20) parse_enum_set: invalid repr ----------------------------------

#[test]
fn test_parse_enum_set_invalid_repr() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "f64"

[enums.TestKind.entries.A]
rust_name = "A"
value = 1
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("illegal repr") || msg.contains("repr"),
        "expected error about illegal repr, got: {msg}"
    );
}

// --- (21) parse_enum_set: empty enums section ---------------------------

#[test]
fn test_parse_enum_set_empty_enums() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("at least one") || msg.contains("empty") || msg.contains("enums"),
        "expected error about empty enums, got: {msg}"
    );
}

// --- (22) parse_enum_set: missing entries section -----------------------

#[test]
fn test_parse_enum_set_missing_entries_section() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("entries") || msg.contains("missing"),
        "expected error about missing entries section, got: {msg}"
    );
}

// --- (23) parse_enum_set: entry value out of u8 range -------------------

#[test]
fn test_parse_enum_set_value_out_of_u8_range() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"

[enums.TestKind.entries.A]
rust_name = "A"
value = 256
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("out of u8 range") || msg.contains("range"),
        "expected error about value out of u8 range, got: {msg}"
    );
}

// --- (24) parse_enum_set: missing rust_name in enum ---------------------

#[test]
fn test_parse_enum_set_missing_enum_rust_name() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
repr = "u8"

[enums.TestKind.entries.A]
rust_name = "A"
value = 1
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("rust_name") || msg.contains("missing"),
        "expected error about missing rust_name, got: {msg}"
    );
}

// --- (25) parse_enum_set: missing value in entry ------------------------

#[test]
fn test_parse_enum_set_entry_missing_value() {
    let toml = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"

[enums.TestKind.entries.A]
rust_name = "A"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("value") || msg.contains("missing"),
        "expected error about missing value, got: {msg}"
    );
}

// --- (26) parse_discriminant: hex value out of u16 range ----------------

#[test]
fn test_parse_discriminant_hex_out_of_u16_range() {
    let toml = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
value = 0x10000
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("out of u16 range") || msg.contains("range"),
        "expected error about value out of u16 range, got: {msg}"
    );
}

// --- (27) parse: missing schema_version --------------------------------

#[test]
fn test_parse_missing_schema_version() {
    let toml = r#"
[meta]
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
value = 0x0001
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("schema_version") || msg.contains("unsupported"),
        "expected error about missing schema_version, got: {msg}"
    );
}

// --- (28) parse: schema_version = 0 ------------------------------------

#[test]
fn test_parse_schema_version_zero() {
    let toml = r#"
[meta]
schema_version = 0
kind = "bit"

[entries.TEST]
rust_name = "Test"
bit = 0
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("schema_version") || msg.contains("unsupported"),
        "expected error about schema_version 0, got: {msg}"
    );
}

// --- (29) parse: schema_version = 2 (future version) -------------------

#[test]
fn test_parse_schema_version_two() {
    let toml = r#"
[meta]
schema_version = 2
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"
[enums.TestKind.entries.A]
rust_name = "A"
value = 1
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("schema_version") || msg.contains("unsupported"),
        "expected error about schema_version 2, got: {msg}"
    );
}

// --- (30) parse: schema_version = -1 -----------------------------------

#[test]
fn test_parse_schema_version_negative() {
    let toml = r#"
[meta]
schema_version = -1
kind = "discriminant"
enum_name = "TestEnum"

[entries.FOO]
rust_name = "Foo"
value = 0x0001
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("schema_version") || msg.contains("unsupported"),
        "expected error about schema_version -1, got: {msg}"
    );
}

// --- (31) parse: unknown kind ---------------------------------------------------

#[test]
fn test_parse_unknown_kind() {
    let toml = r#"
[meta]
schema_version = 1
kind = "invalid"

[entries.TEST]
value = 1
rust_name = "Test"
description = "test"
"#;
    let result = parser::parse_and_validate(toml);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported kind") || msg.contains("kind"),
        "expected error about unsupported kind, got: {msg}"
    );
}
