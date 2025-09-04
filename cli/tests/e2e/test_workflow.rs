//! E2E tests for complete workflows

mod helpers;

use helpers::*;

#[test]
fn test_complete_workflow_new_build_validate() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // The shared project is already created and built
    // Step 1: Verify it was created properly
    assert!(shared_project.file_exists("Cargo.toml"));
    assert!(shared_project.file_exists("src/lib.rs"));

    // Step 2: Build the project again (tests rebuild)
    let output = cli.run_in(shared_project.project_dir(), &["build"]);
    assert_success(&output);

    // Step 3: Validate project
    let output = cli.run_in(shared_project.project_dir(), &["validate"]);
    assert_success(&output);

    // Step 4: Verify outputs exist
    let wasm_path = shared_project
        .project_dir()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("shared_test_project.wasm");
    assert!(wasm_path.exists(), "WASM should exist");

    let did_path = shared_project
        .project_dir()
        .join("src")
        .join("shared-test-project.did");
    assert!(did_path.exists(), "Candid interface should exist");
}

#[test]
fn test_modify_and_rebuild_workflow() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Create a copy since we need to modify it
    let test_project = shared_project.create_copy("modify-rebuild");

    // Modify the code slightly (add a comment)
    let lib_content = test_project.read_file("src/lib.rs");
    let modified = format!("// Modified for testing\n{}", lib_content);
    test_project.write_file("src/lib.rs", &modified);

    // Rebuild should succeed
    let output = cli.run_in(&test_project.project_dir(), &["build"]);
    assert_success(&output);
}

#[test]
fn test_multiple_projects_isolation() {
    let cli = CliRunner::new();
    let shared_project = SharedTestProject::get();

    // Create two copies of the shared project
    let project1 = shared_project.create_copy("project1");
    let project2 = shared_project.create_copy("project2");

    // Modify each project's Cargo.toml to have different names
    let cargo1 = project1.read_file("Cargo.toml");
    let cargo1_modified = cargo1.replace("shared-test-project", "project1");
    project1.write_file("Cargo.toml", &cargo1_modified);

    let cargo2 = project2.read_file("Cargo.toml");
    let cargo2_modified = cargo2.replace("shared-test-project", "project2");
    project2.write_file("Cargo.toml", &cargo2_modified);

    // Build both independently
    let output1 = cli.run_in(&project1.project_dir(), &["build"]);
    let output2 = cli.run_in(&project2.project_dir(), &["build"]);

    assert_success(&output1);
    assert_success(&output2);

    // Verify both have their own artifacts
    assert!(project1
        .project_dir()
        .join("target/wasm32-unknown-unknown/release/project1.wasm")
        .exists());

    assert!(project2
        .project_dir()
        .join("target/wasm32-unknown-unknown/release/project2.wasm")
        .exists());
}
