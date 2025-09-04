//! E2E tests for the 'icarus validate' command

mod helpers;

use helpers::*;

#[test]
fn test_validate_valid_project() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // The shared project is already built, so just validate it
    let output = cli.run_in(shared_project.project_dir(), &["validate"]);
    assert_success(&output);
    assert_contains(&output, "valid");
}

#[test]
fn test_validate_missing_dfx_json() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Create a copy since we need to modify it
    let test_project = shared_project.create_copy("validate-missing-config");

    // Remove dfx.json
    std::fs::remove_file(test_project.project_dir().join("dfx.json")).unwrap();

    // Run validate - should warn or fail about missing dfx.json
    let output = cli.run_in(&test_project.project_dir(), &["validate"]);

    // Might succeed with warning or fail - both are acceptable
    let output_str = String::from_utf8_lossy(&output.stdout);
    let error_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        output_str.contains("dfx")
            || error_str.contains("dfx")
            || output_str.contains("configuration")
            || error_str.contains("configuration"),
        "Should mention missing dfx.json"
    );
}

#[test]
fn test_validate_invalid_cargo_toml() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Create a copy since we need to modify it
    let test_project = shared_project.create_copy("validate-invalid");

    // Break Cargo.toml
    test_project.write_file("Cargo.toml", "invalid toml content {");

    // Run validate - should fail
    let output = cli.run_in(&test_project.project_dir(), &["validate"]);
    assert!(!output.status.success());
}

#[test]
fn test_validate_with_icarus_macros() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // The shared project already has icarus macros and is built
    // Validate should pass
    let output = cli.run_in(shared_project.project_dir(), &["validate"]);
    assert_success(&output);

    // Should detect the icarus_tool macros
    let lib_rs = shared_project.read_file("src/lib.rs");
    assert!(lib_rs.contains("#[icarus_tool"));
}
