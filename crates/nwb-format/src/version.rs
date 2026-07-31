//! NWB format version — single-source-of-truth for the current format version
//! and its Draft compatibility policy.
//!
//! # Design
//!
//! - A single `FormatVersion` struct encodes major/minor.
//! - `compatible(target)` checks whether `self` can read/write `target`.
//! - During Draft (0.x) the only compatible version is the exact current one.
//! - Format 1.0 and later are explicitly rejected until GATE-8 conditions are met.

use core::fmt;

/// Major revision of the NWB binary format.
pub const MAJOR: u16 = 0;
/// Minor revision of the NWB binary format.
pub const MINOR: u16 = 1;

/// Semantic lifecycle of this format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Draft — internal testing only; not for release.
    Draft,
}

/// A parsed NWB format version with its compatibility policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatVersion {
    pub major: u16,
    pub minor: u16,
}

impl FormatVersion {
    /// The current format version (0.1).
    pub const fn current() -> Self {
        FormatVersion {
            major: MAJOR,
            minor: MINOR,
        }
    }

    /// Returns the lifecycle of this format version.
    pub const fn lifecycle(&self) -> Lifecycle {
        Lifecycle::Draft
    }

    /// Returns a human-readable lifecycle banner.
    pub fn lifecycle_banner(&self) -> &'static str {
        "DRAFT — internal testing only; not for release"
    }

    /// Returns `true` if `target` is exactly 0.1 (the only compatible version
    /// during Draft).
    ///
    /// During Draft (0.x) *only* the precise current version is compatible:
    /// - 0.1 → accept
    /// - 0.0 → reject
    /// - 0.2+ → reject
    /// - 1.0+ → reject
    pub fn compatible(&self, target: FormatVersion) -> bool {
        target.major == MAJOR && target.minor == MINOR
    }

    /// Human-readable description of the supported version(s).
    pub fn supported_description(&self) -> String {
        format!("{}.{}", MAJOR, MINOR)
    }

    /// Formats the version as `"major.minor"`.
    pub fn as_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lifecycle::Draft => write!(f, "DRAFT"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TST-VSN-001: exact 0.1 is accepted.
    #[test]
    fn current_version_is_compatible_with_exact_match() {
        let v = FormatVersion::current();
        assert!(v.compatible(FormatVersion { major: 0, minor: 1 }));
    }

    // TST-VSN-002: 0.0 and 0.2 are rejected, and the error message
    // contains both the found version and the supported version.
    #[test]
    fn zero_zero_is_rejected() {
        let v = FormatVersion::current();
        assert!(!v.compatible(FormatVersion { major: 0, minor: 0 }));
    }

    #[test]
    fn zero_two_is_rejected() {
        let v = FormatVersion::current();
        assert!(!v.compatible(FormatVersion { major: 0, minor: 2 }));
    }

    #[test]
    fn error_diagnostic_contains_found_and_supported() {
        let v = FormatVersion::current();
        let found = FormatVersion { major: 0, minor: 0 };
        let supported = v.supported_description();
        let msg = format!(
            "NWB format version {} is not supported; supported version is {}",
            found, supported
        );
        assert!(msg.contains("0.0"));
        assert!(msg.contains("0.1"));
    }

    // TST-VSN-003: 1.0 and future major versions are rejected.
    #[test]
    fn one_zero_is_rejected() {
        let v = FormatVersion::current();
        assert!(!v.compatible(FormatVersion { major: 1, minor: 0 }));
    }

    #[test]
    fn future_major_is_rejected() {
        let v = FormatVersion::current();
        assert!(!v.compatible(FormatVersion { major: 2, minor: 0 }));
        assert!(!v.compatible(FormatVersion {
            major: 99,
            minor: 0
        }));
    }

    // TST-VSN-004: lifecycle banner clearly displays Draft restriction.
    #[test]
    fn lifecycle_banner_displays_draft_restriction() {
        let v = FormatVersion::current();
        let banner = v.lifecycle_banner();
        assert!(banner.contains("DRAFT"));
        assert!(banner.contains("internal testing only"));
        assert!(banner.contains("not for release"));
    }

    // Helper for TST-VSN-005 (CLI test is external; here we just verify
    // the format version display string).
    #[test]
    fn format_version_display() {
        assert_eq!(FormatVersion::current().to_string(), "0.1");
        assert_eq!(FormatVersion::current().lifecycle().to_string(), "DRAFT");
    }
}
