//! E2E tests for complete workflows

mod helpers;

use helpers::*;

#[test]
fn test_complete_workflow_new_build_validate() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("workflow-test");

    // Step 1: Create new project
    let output = cli.run_in(test_project.path(), &["new", "workflow-test"]);
    assert_success(&output);
    assert_contains(&output, "created successfully");

    // Step 2: Build the project
    let output = cli.run_in(&test_project.project_dir(), &["build"]);
    assert_success(&output);

    // Step 3: Validate project (after build so WASM exists)
    let output = cli.run_in(&test_project.project_dir(), &["validate"]);
    assert_success(&output);

    // Step 4: Verify outputs exist
    let wasm_path = test_project
        .project_dir()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("workflow_test.wasm");
    assert!(wasm_path.exists(), "WASM should exist");

    let did_path = test_project
        .project_dir()
        .join("src")
        .join("workflow-test.did");
    assert!(did_path.exists(), "Candid interface should exist");
}

#[test]
fn test_modify_and_rebuild_workflow() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("modify-rebuild");

    // Create and build initially
    cli.run_in(test_project.path(), &["new", "modify-rebuild"]);
    cli.run_in(&test_project.project_dir(), &["build"]);

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

    // Create two separate projects
    let project1 = TestProject::new("project1");
    let project2 = TestProject::new("project2");

    // Create both projects
    cli.run_in(project1.path(), &["new", "project1"]);
    cli.run_in(project2.path(), &["new", "project2"]);

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
