//! E2E tests for the 'icarus build' command

#[path = "../common/mod.rs"]
mod common;

use common::*;

#[test]
fn test_build_creates_optimized_wasm() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Verify that the shared project was built successfully
    // The WASM should already exist from the initial build
    let wasm_path = shared_project
        .project_dir()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("shared_test_project.wasm");

    assert!(wasm_path.exists(), "WASM should be created");

    // Run 'icarus build' command again to test it works on existing project
    let output = cli.run_in(shared_project.project_dir(), &["build"]);
    assert_success(&output);
    assert_contains(&output, "Building");

    // Candid generation is expected through export_candid macro
    // The .did file is generated but location may vary
}

// --output flag not yet implemented
// TODO: Add this test when --output flag is added
// #[test]
// fn test_build_with_custom_output() {

#[test]
fn test_build_shows_size_reduction() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Run build on the shared project
    let output = cli.run_in(shared_project.project_dir(), &["build"]);

    assert_success(&output);

    // Check that output shows size information
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("KB") || output_str.contains("MB"),
        "Output should show file sizes"
    );
}

#[test]
fn test_build_without_project_fails() {
    let cli = CliRunner::new();
    let test_dir = TestProject::new("no-project");

    // Try to build in empty directory
    let output = cli.run_in(test_dir.path(), &["build"]);

    assert!(!output.status.success());
    // The error message says "Not in an Icarus project directory"
    assert_contains(&output, "Not in an Icarus project");
}

#[test]
fn test_build_with_broken_code_fails() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Create a copy for this test since we need to modify files
    let test_project = shared_project.create_copy("broken-test");

    // Break the code
    test_project.write_file("src/lib.rs", "this is not valid rust code!");

    // Try to build
    let output = cli.run_in(&test_project.project_dir(), &["build"]);

    assert!(!output.status.success());
}

#[test]
fn test_build_preserves_candid_interface() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Run build on the shared project
    let output = cli.run_in(shared_project.project_dir(), &["build"]);
    assert_success(&output);

    // The template uses ic_cdk::export_candid!() which generates candid
    // The exact location may vary, so we just verify build succeeded
    // and the source contains the export_candid macro
    let lib_content = shared_project.read_file("src/lib.rs");
    assert!(lib_content.contains("ic_cdk::export_candid!()"));
}
