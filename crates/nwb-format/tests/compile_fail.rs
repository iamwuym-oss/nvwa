//! Compile-fail tests for the NWB format registry.
//!
//! These tests verify that the compiler catches invalid enum definitions
//! at compile time using the `trybuild` crate.

#[test]
fn test_error_id_duplicate_discriminant() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/error_id_duplicate.rs");
}
