#![allow(dead_code)]

//! TOML schema v1 parser -- validates schema header and extracts typed entries.

use crate::schema::{BitEntry, DiscriminantEntry, EnumDef, EnumVariant};

// ---------------------------------------------------------------------------
// Custom deserializer for hex integer values
// ---------------------------------------------------------------------------

/// A u16 that can be deserialized from a hex integer like `0x0001` or a string like `"0x0001"`.
#[derive(Debug, Clone)]
struct HexU16(u16);

impl<'de> serde::Deserialize<'de> for HexU16 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HexU16Visitor;
        impl<'de> serde::de::Visitor<'de> for HexU16Visitor {
            type Value = HexU16;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex integer (e.g. 0x0001) or a hex string (e.g. \"0x0001\")")
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<HexU16, E> {
                if !(0..=0xFFFF).contains(&v) {
                    return Err(E::custom(format!("value {v} out of u16 range")));
                }
                Ok(HexU16(v as u16))
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<HexU16, E> {
                if v > 0xFFFF {
                    return Err(E::custom(format!("value {v} out of u16 range")));
                }
                Ok(HexU16(v as u16))
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<HexU16, E> {
                let s = s.trim();
                if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    u16::from_str_radix(hex, 16)
                        .map(HexU16)
                        .map_err(|e| E::custom(format!("invalid hex value \"{s}\": {e}")))
                } else {
                    s.parse::<u16>()
                        .map(HexU16)
                        .map_err(|e| E::custom(format!("invalid value \"{s}\": {e}")))
                }
            }
        }
        deserializer.deserialize_any(HexU16Visitor)
    }
}

// ---------------------------------------------------------------------------
// Serde structures for raw TOML deserialization
// ---------------------------------------------------------------------------

/// Top-level structure for discriminant-kind TOML.
#[derive(serde::Deserialize)]
struct DiscriminantToml {
    meta: DiscriminantMeta,
    entries: std::collections::BTreeMap<String, DiscriminantEntryToml>,
}

#[derive(serde::Deserialize)]
struct DiscriminantMeta {
    schema_version: Option<i64>,
    kind: Option<String>,
    enum_name: Option<String>,
}

#[derive(serde::Deserialize)]
struct DiscriminantEntryToml {
    rust_name: Option<String>,
    value: Option<HexU16>,
    description: Option<String>,
}

/// Top-level structure for bit-kind TOML.
#[derive(serde::Deserialize)]
struct BitToml {
    meta: Meta,
    entries: std::collections::BTreeMap<String, BitEntryToml>,
}

#[derive(serde::Deserialize)]
struct BitEntryToml {
    rust_name: Option<String>,
    bit: Option<i64>,
    description: Option<String>,
}

/// Top-level structure for enum_set-kind TOML.
#[derive(serde::Deserialize)]
struct EnumSetToml {
    meta: Meta,
    enums: std::collections::BTreeMap<String, EnumDescrToml>,
}

#[derive(serde::Deserialize)]
struct EnumDescrToml {
    rust_name: Option<String>,
    repr: Option<String>,
    entries: Option<std::collections::BTreeMap<String, EnumEntryToml>>,
}

#[derive(serde::Deserialize)]
struct EnumEntryToml {
    rust_name: Option<String>,
    value: Option<i64>,
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct Meta {
    schema_version: Option<i64>,
    kind: Option<String>,
}

// ---------------------------------------------------------------------------
// Parsed representation
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParsedRegistry {
    Discriminant(DiscriminantRegistry),
    Bit(Vec<BitEntry>),
    EnumSet(Vec<EnumDef>),
}

/// Discriminant entries with the parent enum name.
#[derive(Debug, Clone)]
pub struct DiscriminantRegistry {
    pub enum_name: String,
    pub entries: Vec<DiscriminantEntry>,
}

impl ParsedRegistry {
    #[allow(dead_code)]
    pub fn kind(&self) -> &str {
        match self {
            ParsedRegistry::Discriminant(_) => "discriminant",
            ParsedRegistry::Bit(_) => "bit",
            ParsedRegistry::EnumSet(_) => "enum_set",
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParseError {
    Toml(String),
    Validation(String),
    SemanticValidation(String),
    Io(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Toml(msg) => write!(f, "TOML parse error: {msg}"),
            ParseError::Validation(msg) => write!(f, "validation error: {msg}"),
            ParseError::SemanticValidation(msg) => write!(f, "semantic error: {msg}"),
            ParseError::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

/// Helper: extract [meta] section values from TOML without knowing the full type.
fn extract_meta(toml_str: &str) -> Result<Meta, ParseError> {
    let value: toml::Value =
        toml::from_str(toml_str).map_err(|e| ParseError::Toml(e.to_string()))?;

    let meta_table = value
        .get("meta")
        .and_then(|v| v.as_table())
        .ok_or_else(|| ParseError::Validation("missing [meta] section".to_string()))?;

    let schema_version = meta_table
        .get("schema_version")
        .and_then(|v| v.as_integer());

    let kind = meta_table
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Meta {
        schema_version,
        kind,
    })
}

// ---------------------------------------------------------------------------
// Parser function
// ---------------------------------------------------------------------------

/// Parse a schema v1 TOML string and return the validated registry.
pub fn parse_and_validate(toml_str: &str) -> Result<ParsedRegistry, ParseError> {
    let meta = extract_meta(toml_str)?;

    let schema_version = meta.schema_version.unwrap_or(0);
    if schema_version != 1 {
        return Err(ParseError::Validation(format!(
            "unsupported schema_version: {schema_version} (expected 1)"
        )));
    }

    let kind = meta.kind.as_deref().unwrap_or("").to_string();
    match kind.as_str() {
        "discriminant" => parse_discriminant(toml_str),
        "bit" => parse_bit(toml_str),
        "enum_set" => parse_enum_set(toml_str),
        other => Err(ParseError::Validation(format!(
            "unknown kind: \"{other}\" (expected \"discriminant\", \"bit\", or \"enum_set\")"
        ))),
    }
}

fn parse_discriminant(toml_str: &str) -> Result<ParsedRegistry, ParseError> {
    let toml_data: DiscriminantToml =
        toml::from_str(toml_str).map_err(|e| ParseError::Toml(e.to_string()))?;

    let enum_name = toml_data.meta.enum_name.ok_or_else(|| {
        ParseError::Validation(
            "discriminant meta is missing required field \"enum_name\"".to_string(),
        )
    })?;

    let mut entries: Vec<DiscriminantEntry> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for (canonical_name, et) in &toml_data.entries {
        let rust_name = et.rust_name.as_deref().unwrap_or("");
        let desc = et.description.as_deref().unwrap_or("");

        if rust_name.is_empty() {
            missing.push(format!("{canonical_name}.rust_name"));
        }

        let value = match &et.value {
            Some(HexU16(v)) => *v,
            None => {
                missing.push(format!("{canonical_name}.value"));
                0
            }
        };

        entries.push(DiscriminantEntry {
            canonical_name: canonical_name.clone(),
            rust_name: rust_name.to_string(),
            value,
            description: desc.to_string(),
        });
    }

    if !missing.is_empty() {
        return Err(ParseError::Validation(format!(
            "missing required fields: {}",
            missing.join(", ")
        )));
    }

    match crate::schema::validate_discriminant_entries(&entries) {
        Ok(()) => Ok(ParsedRegistry::Discriminant(DiscriminantRegistry {
            enum_name,
            entries,
        })),
        Err(errors) => Err(ParseError::Validation(
            crate::schema::format_validation_errors(&errors),
        )),
    }
}

fn parse_bit(toml_str: &str) -> Result<ParsedRegistry, ParseError> {
    let toml_data: BitToml =
        toml::from_str(toml_str).map_err(|e| ParseError::Toml(e.to_string()))?;

    let mut entries: Vec<BitEntry> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for (canonical_name, et) in &toml_data.entries {
        let rust_name = et.rust_name.as_deref().unwrap_or("");
        let bit = et.bit;
        let description = et.description.as_deref().unwrap_or("");

        if rust_name.is_empty() {
            missing.push(format!("{canonical_name}.rust_name"));
        }
        if bit.is_none() {
            missing.push(format!("{canonical_name}.bit"));
        }

        let bit_val = bit.unwrap_or(0);
        if !(0..=255).contains(&bit_val) {
            return Err(ParseError::Validation(format!(
                "bit value {bit_val} out of u8 range in entry \"{canonical_name}\""
            )));
        }

        entries.push(BitEntry {
            canonical_name: canonical_name.clone(),
            rust_name: rust_name.to_string(),
            bit: bit_val as u8,
            description: description.to_string(),
        });
    }

    if !missing.is_empty() {
        return Err(ParseError::Validation(format!(
            "missing required fields: {}",
            missing.join(", ")
        )));
    }

    match crate::schema::validate_bit_entries(&entries) {
        Ok(()) => Ok(ParsedRegistry::Bit(entries)),
        Err(errors) => Err(ParseError::Validation(
            crate::schema::format_validation_errors(&errors),
        )),
    }
}

fn parse_enum_set(toml_str: &str) -> Result<ParsedRegistry, ParseError> {
    let toml_data: EnumSetToml =
        toml::from_str(toml_str).map_err(|e| ParseError::Toml(e.to_string()))?;

    if toml_data.enums.is_empty() {
        return Err(ParseError::Validation(
            "enum_set must contain at least one [enums.*] section".to_string(),
        ));
    }

    let mut enums: Vec<EnumDef> = Vec::new();

    for (enum_canonical, et) in &toml_data.enums {
        let rust_name = et.rust_name.as_deref().unwrap_or("");
        let repr = et.repr.as_deref().unwrap_or("");

        if rust_name.is_empty() {
            return Err(ParseError::Validation(format!(
                "enum \"{enum_canonical}\" missing required field \"rust_name\""
            )));
        }
        match repr {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {}
            "" => {
                return Err(ParseError::Validation(format!(
                    "enum \"{enum_canonical}\" missing required field \"repr\""
                )));
            }
            other => {
                return Err(ParseError::Validation(format!(
                    "enum \"{enum_canonical}\" illegal repr \"{other}\" (expected u8, u16, etc.)"
                )));
            }
        }

        let entries_toml = et.entries.as_ref().ok_or_else(|| {
            ParseError::Validation(format!(
                "enum \"{enum_canonical}\" missing [enums.{enum_canonical}.entries] section"
            ))
        })?;

        let mut variants: Vec<EnumVariant> = Vec::new();
        for (var_canonical, vt) in entries_toml {
            let var_rust_name = vt.rust_name.as_deref().unwrap_or("");
            let var_value = vt.value;
            let var_desc = vt.description.as_deref().unwrap_or("");

            if var_rust_name.is_empty() {
                return Err(ParseError::Validation(format!(
                    "enum \"{enum_canonical}\" entry \"{var_canonical}\" missing rust_name"
                )));
            }
            if var_value.is_none() {
                return Err(ParseError::Validation(format!(
                    "enum \"{enum_canonical}\" entry \"{var_canonical}\" missing value"
                )));
            }

            let val = var_value.unwrap();
            if !(0..=255).contains(&val) {
                return Err(ParseError::Validation(format!(
                    "enum \"{enum_canonical}\" entry \"{var_canonical}\" value {val} out of u8 range"
                )));
            }

            variants.push(EnumVariant {
                canonical_name: var_canonical.clone(),
                rust_name: var_rust_name.to_string(),
                value: val as u8,
                description: var_desc.to_string(),
            });
        }

        let def = EnumDef {
            canonical_name: enum_canonical.clone(),
            rust_name: rust_name.to_string(),
            repr: repr.to_string(),
            entries: variants,
        };

        match crate::schema::validate_enum_def(&def) {
            Ok(()) => enums.push(def),
            Err(errors) => {
                return Err(ParseError::SemanticValidation(
                    crate::schema::format_validation_errors(&errors),
                ));
            }
        }
    }

    Ok(ParsedRegistry::EnumSet(enums))
}
