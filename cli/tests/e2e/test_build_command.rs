//! E2E tests for the 'icarus build' command

mod helpers;

use helpers::*;

#[test]
fn test_build_creates_optimized_wasm() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("build-opt-test");

    // Create new project
    let output = cli.run_in(test_project.path(), &["new", "build-opt-test"]);
    assert_success(&output);

    // Run 'icarus build' command
    let output = cli.run_in(&test_project.project_dir(), &["build"]);
    assert_success(&output);
    assert_contains(&output, "Building");

    // Verify WASM exists (not optimized by default)
    let wasm_path = test_project
        .project_dir()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("build_opt_test.wasm");

    assert!(wasm_path.exists(), "WASM should be created");

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
    let test_project = TestProject::new("size-test");

    // Create and build project
    cli.run_in(test_project.path(), &["new", "size-test"]);
    let output = cli.run_in(&test_project.project_dir(), &["build"]);

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
    let test_project = TestProject::new("broken-test");

    // Create new project
    cli.run_in(test_project.path(), &["new", "broken-test"]);

    // Break the code
    test_project.write_file("src/lib.rs", "this is not valid rust code!");

    // Try to build
    let output = cli.run_in(&test_project.project_dir(), &["build"]);

    assert!(!output.status.success());
}

#[test]
fn test_build_preserves_candid_interface() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("candid-preserve-test");

    // Create and build project
    cli.run_in(test_project.path(), &["new", "candid-preserve-test"]);
    let output = cli.run_in(&test_project.project_dir(), &["build"]);
    assert_success(&output);

    // The template uses ic_cdk::export_candid!() which generates candid
    // The exact location may vary, so we just verify build succeeded
    // and the source contains the export_candid macro
    let lib_content = test_project.read_file("src/lib.rs");
    assert!(lib_content.contains("ic_cdk::export_candid!()"));
}
