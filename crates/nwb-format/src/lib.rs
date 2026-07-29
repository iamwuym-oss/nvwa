//! nwb-format: Nüwa Backup NWB binary format structures, encoding, and parsing.
//!
//! This crate provides the fundamental type definitions, codecs, and format-level
//! utilities for the NWB storage format. It has no platform-specific dependencies
//! and must compile on both Windows and Linux.

/// Version of this crate — matches the NWB format version it implements.
pub const NWB_FORMAT_VERSION: &str = "0.1.0-dev";

pub mod registry;
pub mod version;

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_defined() {
        assert!(!super::NWB_FORMAT_VERSION.is_empty());
    }
}
