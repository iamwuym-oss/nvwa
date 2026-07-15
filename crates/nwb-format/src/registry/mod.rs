//! Registry: format constants, enumerations, and capabilities.
//!
//! This module is the single source of truth for NWB binary format constants
//! that appear on the wire: record type discriminants, feature capability
//! bits, and header enumeration values.

pub mod feature_bit;
pub mod header_enums;
pub mod record_type;
