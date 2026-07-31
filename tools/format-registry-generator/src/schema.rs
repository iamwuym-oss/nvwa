#![allow(dead_code)]

//! Schema v1 validation logic for Format Registry TOML files.

use std::collections::HashSet;

/// Describes a single entry in a "discriminant"-kind TOML file.
#[derive(Debug, Clone)]
pub struct DiscriminantEntry {
    pub canonical_name: String,
    pub rust_name: String,
    /// Hex string such as "0x0001".
    pub value: u16,
    pub description: String,
}

/// Describes a single entry in a "bit"-kind TOML file.
#[derive(Debug, Clone)]
pub struct BitEntry {
    pub canonical_name: String,
    pub rust_name: String,
    pub bit: u8,
    pub description: String,
}

/// Describes a single variant in an enum_set entry.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub canonical_name: String,
    pub rust_name: String,
    pub value: u8,
    pub description: String,
}

/// Describes one enum in an "enum_set"-kind TOML file.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub canonical_name: String,
    pub rust_name: String,
    pub repr: String,
    pub entries: Vec<EnumVariant>,
}

// ---------------------------------------------------------------------------
// Semantic validation (beyond type-level checks)
// ---------------------------------------------------------------------------

/// Semantic errors found during validation.
#[derive(Debug)]
pub enum ValidationError {
    DuplicateValue(String, String, u64),
    BitOutOfRange(String, u8),
    ReservedConflict(String, u64),
    MissingRequiredField(String, String),
    DuplicateCanonicalName(String),
    DuplicateRustName(String),
    DuplicateEnumValue(String, String, u8),
}

/// Check a list of discriminant entries for semantic errors.
pub fn validate_discriminant_entries(
    entries: &[DiscriminantEntry],
) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut seen_values: HashSet<u16> = HashSet::new();
    let mut seen_canonical: HashSet<&str> = HashSet::new();
    let mut seen_rust: HashSet<&str> = HashSet::new();

    for e in entries {
        // Duplicate canonical name
        if !seen_canonical.insert(&e.canonical_name) {
            errors.push(ValidationError::DuplicateCanonicalName(
                e.canonical_name.clone(),
            ));
        }
        // Duplicate rust_name
        if !seen_rust.insert(&e.rust_name) {
            errors.push(ValidationError::DuplicateRustName(e.rust_name.clone()));
        }
        // Reserved 0x0000 check
        if e.value == 0x0000 && e.canonical_name != "INVALID" {
            errors.push(ValidationError::ReservedConflict(
                e.canonical_name.clone(),
                e.value as u64,
            ));
        }
        // Duplicate value
        if !seen_values.insert(e.value) {
            errors.push(ValidationError::DuplicateValue(
                e.canonical_name.clone(),
                format!("0x{:04X}", e.value),
                e.value as u64,
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check a list of bit entries for semantic errors.
pub fn validate_bit_entries(entries: &[BitEntry]) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut seen_bits: HashSet<u8> = HashSet::new();
    let mut seen_canonical: HashSet<&str> = HashSet::new();
    let mut seen_rust: HashSet<&str> = HashSet::new();

    for e in entries {
        // Duplicate canonical name
        if !seen_canonical.insert(&e.canonical_name) {
            errors.push(ValidationError::DuplicateCanonicalName(
                e.canonical_name.clone(),
            ));
        }
        // Duplicate rust_name
        if !seen_rust.insert(&e.rust_name) {
            errors.push(ValidationError::DuplicateRustName(e.rust_name.clone()));
        }
        // Bit out of range
        if e.bit > 63 {
            errors.push(ValidationError::BitOutOfRange(
                e.canonical_name.clone(),
                e.bit,
            ));
        }
        // Duplicate bit
        if !seen_bits.insert(e.bit) {
            errors.push(ValidationError::DuplicateValue(
                e.canonical_name.clone(),
                e.bit.to_string(),
                e.bit as u64,
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check an enum definition for semantic errors.
pub fn validate_enum_def(def: &EnumDef) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut seen_values: HashSet<u8> = HashSet::new();
    let mut seen_variant_canonical: HashSet<&str> = HashSet::new();
    let mut seen_variant_rust: HashSet<&str> = HashSet::new();

    for v in &def.entries {
        // Duplicate canonical name
        if !seen_variant_canonical.insert(&v.canonical_name) {
            errors.push(ValidationError::DuplicateCanonicalName(
                v.canonical_name.clone(),
            ));
        }
        // Duplicate rust_name
        if !seen_variant_rust.insert(&v.rust_name) {
            errors.push(ValidationError::DuplicateRustName(v.rust_name.clone()));
        }
        // Duplicate value
        if !seen_values.insert(v.value) {
            errors.push(ValidationError::DuplicateEnumValue(
                def.canonical_name.clone(),
                v.canonical_name.clone(),
                v.value,
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Format validation errors for display.
pub fn format_validation_errors(errors: &[ValidationError]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for e in errors {
        match e {
            ValidationError::DuplicateValue(name, val, _) => {
                lines.push(format!("  duplicate value {val} in entry \"{name}\""));
            }
            ValidationError::BitOutOfRange(name, bit) => {
                lines.push(format!(
                    "  bit {bit} out of range (0-63) in entry \"{name}\""
                ));
            }
            ValidationError::ReservedConflict(name, val) => {
                lines.push(format!(
                    "  entry \"{name}\" uses reserved value 0x{val:04X}"
                ));
            }
            ValidationError::MissingRequiredField(name, field) => {
                lines.push(format!(
                    "  entry \"{name}\" missing required field \"{field}\""
                ));
            }
            ValidationError::DuplicateCanonicalName(name) => {
                lines.push(format!("  duplicate canonical name \"{name}\""));
            }
            ValidationError::DuplicateRustName(name) => {
                lines.push(format!("  duplicate rust_name \"{name}\""));
            }
            ValidationError::DuplicateEnumValue(enum_name, variant, val) => {
                lines.push(format!(
                    "  enum \"{enum_name}\": variant \"{variant}\" has duplicate value {val}"
                ));
            }
        }
    }
    lines.join("\n")
}
