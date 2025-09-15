//! UI tests for icarus-derive procedural macros
//!
//! These tests verify that the derive macros compile correctly with valid inputs
//! and produce helpful error messages with invalid inputs.

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // Test cases that should compile successfully
    t.pass("tests/ui/pass/*.rs");

    // Test cases that should fail with helpful error messages
    t.compile_fail("tests/ui/fail/*.rs");
}
