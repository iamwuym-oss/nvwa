//! Registry: format constants, enumerations, and capabilities.
//!
//! This module is the single source of truth for NWB binary format constants
//! that appear on the wire: record type discriminants, feature capability
//! bits, header enumeration values, and error identifiers.

pub mod error_id;
pub mod feature_bit;
pub mod header_enums;
pub mod record_type;
