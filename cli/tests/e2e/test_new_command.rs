//! E2E tests for the 'icarus new' command

#[path = "../common/mod.rs"]
mod common;

use common::*;

#[test]
fn test_new_creates_project_structure() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("test-app");

    // Run 'icarus new' command
    let output = cli.run_in(test_project.path(), &["new", "test-app"]);
    assert_success(&output);
    assert_contains(&output, "created successfully");

    // Verify project structure
    assert!(test_project.file_exists("Cargo.toml"));
    assert!(test_project.file_exists("src/lib.rs"));
    assert!(test_project.file_exists(".gitignore"));
    assert!(test_project.file_exists("dfx.json"));
    assert!(test_project.file_exists("README.md"));

    // Verify Cargo.toml content
    let cargo_toml = test_project.read_file("Cargo.toml");
    assert!(cargo_toml.contains("name = \"test-app\""));
    assert!(cargo_toml.contains("icarus = "));
    assert!(cargo_toml.contains("crate-type = [\"cdylib\"]"));

    // Verify src/lib.rs contains the basic memory template
    let lib_rs = test_project.read_file("src/lib.rs");
    assert!(lib_rs.contains("//! Basic Memory Server"));
    assert!(lib_rs.contains("use icarus::prelude::*"));
    assert!(lib_rs.contains("stable_storage!"));
    assert!(lib_rs.contains("MEMORIES: StableBTreeMap<String, MemoryEntry, Memory>"));
    assert!(lib_rs.contains("#[icarus_tool"));
    assert!(lib_rs.contains("pub fn memorize"));
    assert!(lib_rs.contains("pub fn recall"));
    assert!(lib_rs.contains("pub fn list"));
    assert!(lib_rs.contains("pub fn search_by_tag"));
    assert!(lib_rs.contains("pub fn forget"));
    assert!(lib_rs.contains("pub fn count"));
}

#[test]
fn test_new_project_builds_successfully() {
    // Use the shared project to verify it was built successfully
    // This avoids building a new project just to test if it builds
    let shared_project = SharedTestProject::get();

    // Verify WASM output exists
    let wasm_path = shared_project
        .project_dir()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("shared_test_project.wasm");

    assert!(wasm_path.exists(), "WASM file should be generated");

    // Also verify the project structure is correct
    assert!(shared_project.file_exists("Cargo.toml"));
    assert!(shared_project.file_exists("src/lib.rs"));
}

#[test]
fn test_new_with_existing_directory_fails() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("existing-app");

    // Create directory first
    std::fs::create_dir_all(test_project.project_dir()).unwrap();

    // Try to create project in existing directory
    let output = cli.run_in(test_project.path(), &["new", "existing-app"]);
    assert!(!output.status.success());
    assert_contains(&output, "already exists");
}

// TODO: Re-enable this test when --template flag is implemented for custom project templates
// #[test]
// fn test_new_with_custom_template() {
//     let cli = CliRunner::new();
//     let test_project = TestProject::new("custom-template");
//
//     // Test with --template flag (even though we only have default for now)
//     let output = cli.run_in(test_project.path(), &["new", "custom-template", "--template", "default"]);
//     assert_success(&output);
//
//     // Verify it still creates the default template
//     let lib_rs = test_project.read_file("src/lib.rs");
//     assert!(lib_rs.contains("Memento"));
// }

#[test]
fn test_new_creates_valid_candid_interface() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("candid-test");

    // Create project (don't build - we just check the source)
    let output = cli.run_in(test_project.path(), &["new", "candid-test"]);
    assert_success(&output);

    // Check if .did file was created
    // (we now create a minimal .did file directly)
    let project_name = "candid-test";
    assert!(test_project.file_exists(&format!("src/{}.did", project_name)));
}
