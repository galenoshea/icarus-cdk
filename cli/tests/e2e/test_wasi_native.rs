//! E2E tests for WASI-Native architecture

#[path = "../common/mod.rs"]
mod common;

use common::*;

#[test]
fn test_build_uses_wasi_by_default() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("wasi-default-test");

    // Create a new project
    let output = cli.run_in(test_project.path(), &["new", "wasi-default-test"]);
    assert_success(&output);

    test_project.create_cargo_config_override();

    // Create a minimal working lib.rs to test WASI compilation
    let minimal_lib = r#"//! Minimal test project for WASI compilation
use icarus::prelude::*;

#[derive(Default)]
pub struct TestService;

#[icarus_tools]
impl IcarusToolProvider for TestService {

    #[tool("Test tool that returns a greeting")]
    #[query]
    async fn greet(name: String) -> Result<String, String> {
        Ok(format!("Hello, {}!", name))
    }
}
"#;

    test_project.write_file("src/lib.rs", minimal_lib);

    // Build the project (should use WASI by default)
    let output = cli.run_in(&test_project.project_dir(), &["build"]);

    // The key test is that it attempts to build with WASI target
    let output_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}\n{}", output_str, stderr_str);

    // Verify it's using WASI target (this is the main point of the test)
    assert!(
        combined_output.contains("wasm32-wasip1") || combined_output.contains("WASI"),
        "Build should use WASI target. Output: {}",
        combined_output
    );

    // If build succeeds, verify artifacts
    if output.status.success() {
        // Verify WASI WASM was created
        let wasi_wasm_path = test_project
            .project_dir()
            .join("target")
            .join("wasm32-wasip1")
            .join("release")
            .join("wasi_default_test.wasm");

        assert!(
            wasi_wasm_path.exists(),
            "WASI WASM should be created: {:?}",
            wasi_wasm_path
        );

        // Verify final IC-compatible WASM was created (converted from WASI)
        let final_wasm_path = test_project
            .project_dir()
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("wasi_default_test.wasm");

        assert!(
            final_wasm_path.exists(),
            "Final IC-compatible WASM should be created: {:?}",
            final_wasm_path
        );
    }
}

#[test]
fn test_build_with_pure_wasm_flag() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("pure-wasm-test");

    // Create a new project
    let output = cli.run_in(test_project.path(), &["new", "pure-wasm-test"]);
    assert_success(&output);

    test_project.create_cargo_config_override();

    // Create the same minimal working lib.rs
    let minimal_lib = r#"//! Minimal test project for pure WASM compilation
use icarus::prelude::*;

#[derive(Default)]
pub struct TestService;

#[icarus_tools]
impl IcarusToolProvider for TestService {

    #[tool("Test tool that returns a greeting")]
    #[query]
    async fn greet(name: String) -> Result<String, String> {
        Ok(format!("Hello, {}!", name))
    }
}
"#;

    test_project.write_file("src/lib.rs", minimal_lib);

    // Build the project with --pure-wasm flag
    let output = cli.run_in(&test_project.project_dir(), &["build", "--pure-wasm"]);

    // The key test is that it uses pure WASM target
    let output_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}\n{}", output_str, stderr_str);

    // Verify it's using pure WASM target and NOT WASI
    assert!(
        combined_output.contains("wasm32-unknown-unknown")
            && !combined_output.contains("wasm32-wasip1"),
        "Build should use pure WASM target (wasm32-unknown-unknown) and not WASI. Output: {}",
        combined_output
    );

    // If build succeeds, verify artifacts
    if output.status.success() {
        // Verify pure WASM was created directly
        let pure_wasm_path = test_project
            .project_dir()
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("pure_wasm_test.wasm");

        assert!(
            pure_wasm_path.exists(),
            "Pure WASM should be created: {:?}",
            pure_wasm_path
        );

        // Verify WASI WASM was NOT created
        let wasi_wasm_path = test_project
            .project_dir()
            .join("target")
            .join("wasm32-wasip1")
            .join("release")
            .join("pure_wasm_test.wasm");

        assert!(
            !wasi_wasm_path.exists(),
            "WASI WASM should not be created with --pure-wasm: {:?}",
            wasi_wasm_path
        );
    }
}

#[test]
fn test_wasi_target_selection() {
    let cli = CliRunner::new();
    let test_project = TestProject::new("target-test");

    // Create a new project
    let output = cli.run_in(test_project.path(), &["new", "target-test"]);
    assert_success(&output);

    test_project.create_cargo_config_override();

    // Create the same minimal working lib.rs
    let minimal_lib = r#"//! Minimal test project for target selection
use icarus::prelude::*;

#[derive(Default)]
pub struct TestService;

#[icarus_tools]
impl IcarusToolProvider for TestService {

    #[tool("Test tool that returns a greeting")]
    #[query]
    async fn greet(name: String) -> Result<String, String> {
        Ok(format!("Hello, {}!", name))
    }
}
"#;

    test_project.write_file("src/lib.rs", minimal_lib);

    // Test 1: Default build should use WASI
    let output_default = cli.run_in(&test_project.project_dir(), &["build"]);
    let default_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output_default.stdout),
        String::from_utf8_lossy(&output_default.stderr)
    );

    assert!(
        default_output.contains("wasm32-wasip1") || default_output.contains("WASI"),
        "Default build should use WASI target. Output: {}",
        default_output
    );

    // Test 2: --pure-wasm flag should use pure WASM
    let output_pure = cli.run_in(&test_project.project_dir(), &["build", "--pure-wasm"]);
    let pure_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output_pure.stdout),
        String::from_utf8_lossy(&output_pure.stderr)
    );

    assert!(
        pure_output.contains("wasm32-unknown-unknown") && !pure_output.contains("wasm32-wasip1"),
        "Pure WASM build should use wasm32-unknown-unknown target. Output: {}",
        pure_output
    );
}
