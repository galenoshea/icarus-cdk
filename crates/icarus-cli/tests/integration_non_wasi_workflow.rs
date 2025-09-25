//! Integration tests for non-WASI workflow end-to-end functionality
//!
//! Tests the complete non-WASI (pure WASM) workflow from project creation to build verification

mod common;

use common::{assert_success, CliRunner, TestProject};

/// Test complete non-WASI project creation workflow
#[test]
fn test_non_wasi_project_creation_end_to_end() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-e2e");

    // Run icarus new WITHOUT --wasi flag (default behavior)
    let output = runner.run_in(project.path(), &["new", "test-non-wasi-e2e"]);

    // Verify command succeeded
    assert_success(&output);

    // Verify project directory was created
    assert!(
        project.project_dir().exists(),
        "Project directory should be created: {:?}",
        project.project_dir()
    );

    // Verify essential files exist
    assert!(project.file_exists("Cargo.toml"), "Cargo.toml should exist");
    assert!(project.file_exists("src/lib.rs"), "src/lib.rs should exist");
    assert!(project.file_exists("dfx.json"), "dfx.json should exist");

    // Verify NO WASI-specific content in Cargo.toml
    let cargo_content = project.read_file("Cargo.toml");
    assert!(
        !cargo_content.contains("ic-wasi-polyfill"),
        "Cargo.toml should NOT contain ic-wasi-polyfill dependency: {}",
        cargo_content
    );
    assert!(
        !cargo_content.contains("wasi = ["),
        "Cargo.toml should NOT define WASI feature: {}",
        cargo_content
    );
    assert!(
        !cargo_content.contains("default = [\"wasi\"]"),
        "Cargo.toml should NOT have WASI in default features: {}",
        cargo_content
    );

    // Verify NO WASI-specific content in lib.rs
    let lib_content = project.read_file("src/lib.rs");
    assert!(
        !lib_content.contains("wasi_init!()"),
        "lib.rs should NOT contain wasi_init!() call: {}",
        lib_content
    );
    assert!(
        !lib_content.contains("WASI"),
        "lib.rs should NOT mention WASI: {}",
        lib_content
    );
}

/// Test non-WASI project builds with pure WASM target
#[test]
fn test_non_wasi_project_builds_pure_wasm() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-build");

    // Create non-WASI project
    let output = runner.run_in(project.path(), &["new", "test-non-wasi-build"]);
    assert_success(&output);

    // Create cargo config override to use local dependencies
    project.create_cargo_config_override();

    // Test that build command uses pure WASM target
    let build_output = runner.run_in(&project.project_dir(), &["build"]);

    // Build should succeed (or at least start the process correctly)
    let stdout = String::from_utf8_lossy(&build_output.stdout);
    let stderr = String::from_utf8_lossy(&build_output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // Verify pure WASM target selection (should mention unknown-unknown, not wasip1)
    assert!(
        combined_output.contains("pure WASM")
            || combined_output.contains("wasm32-unknown-unknown")
            || combined_output.contains("simple mode"),
        "Build output should mention pure WASM or simple mode: {}",
        combined_output
    );

    // Should NOT mention WASI-related targets
    assert!(
        !combined_output.contains("wasip1") && !combined_output.contains("WASI support"),
        "Build output should NOT mention WASI or wasip1 for non-WASI project: {}",
        combined_output
    );
}

/// Test non-WASI project structure and content
#[test]
fn test_non_wasi_project_structure() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-structure");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-structure"],
    );
    assert_success(&output);

    // Verify project structure is optimized for simple canisters
    let cargo_content = project.read_file("Cargo.toml");

    // Should have core IC dependencies
    assert!(
        cargo_content.contains("ic-cdk"),
        "Should have ic-cdk dependency: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("candid"),
        "Should have candid dependency: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("serde"),
        "Should have serde dependency: {}",
        cargo_content
    );

    // Should NOT have WASI-related dependencies
    assert!(
        !cargo_content.contains("ic-wasi-polyfill"),
        "Should NOT have WASI polyfill: {}",
        cargo_content
    );

    // Should have proper crate type
    assert!(
        cargo_content.contains("crate-type = [\"cdylib\"]"),
        "Should have cdylib crate type: {}",
        cargo_content
    );

    // Verify lib.rs content is minimal and focused
    let lib_content = project.read_file("src/lib.rs");

    // Should have basic canister structure
    assert!(
        lib_content.contains("use icarus::prelude::*"),
        "Should import icarus prelude: {}",
        lib_content
    );

    // Should NOT have WASI initialization
    assert!(
        !lib_content.contains("wasi_init"),
        "Should NOT have WASI initialization: {}",
        lib_content
    );
}

/// Test non-WASI project with tests enabled
#[test]
fn test_non_wasi_project_with_tests() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-tests");

    // Create non-WASI project with tests
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-tests", "--with-tests"],
    );
    assert_success(&output);

    // Verify test files are created
    assert!(
        project.file_exists("tests/test_basic.rs"),
        "Test file should be created"
    );

    // Verify test content is appropriate for non-WASI project
    let test_content = project.read_file("tests/test_basic.rs");

    // Tests should be simple canister tests, not WASI-specific
    assert!(
        !test_content.contains("wasi"),
        "Test content should not reference WASI: {}",
        test_content
    );
}

/// Test non-WASI project optimization settings
#[test]
fn test_non_wasi_project_optimization() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-optimization");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-optimization"],
    );
    assert_success(&output);

    // Verify optimization settings for pure WASM
    let cargo_content = project.read_file("Cargo.toml");

    // Should have release optimizations
    assert!(
        cargo_content.contains("[profile.release]"),
        "Should have release profile optimizations: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("lto = true"),
        "Should enable LTO: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("strip = \"debuginfo\""),
        "Should strip debug info: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("codegen-units = 1"),
        "Should use single codegen unit: {}",
        cargo_content
    );
}

/// Test non-WASI project deployment configuration
#[test]
fn test_non_wasi_project_deployment() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-deployment");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-deployment"],
    );
    assert_success(&output);

    // Verify dfx.json is configured for simple deployment
    let dfx_content = project.read_file("dfx.json");

    // Should have custom canister type for WASM builds
    assert!(
        dfx_content.contains("\"type\": \"custom\""),
        "Should have custom canister type: {}",
        dfx_content
    );

    // Should NOT have WASI-specific deployment settings
    assert!(
        !dfx_content.contains("wasi"),
        "Should not have WASI-specific deployment config: {}",
        dfx_content
    );
}

/// Test non-WASI project error conditions
#[test]
fn test_non_wasi_project_error_handling() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-errors");

    // Test with invalid project name
    let output = runner.run_in(
        project.path(),
        &["new", "invalid-name-with-special@chars"],
    );

    // Should fail with informative error
    assert!(
        !output.status.success(),
        "Should fail with invalid project name"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("name"),
        "Error should mention invalid name: {}",
        stderr
    );
}

/// Test non-WASI project consistency across different scenarios
#[test]
fn test_non_wasi_project_consistency() {
    let runner = CliRunner::new();

    // Create multiple non-WASI projects with different settings
    let scenarios = vec![
        ("basic", vec!["new", "basic"]),
        (
            "with-tests",
            vec!["new", "with-tests", "--with-tests"],
        ),
        ("no-tests", vec!["new", "no-tests"]),
    ];

    for (name, args) in scenarios {
        let project = TestProject::new(&format!("test-non-wasi-{}", name));

        let output = runner.run_in(project.path(), &args);
        assert_success(&output);

        // All scenarios should have consistent non-WASI structure
        let cargo_content = project.read_file("Cargo.toml");
        let lib_content = project.read_file("src/lib.rs");

        // Consistent absence of WASI content
        assert!(
            !cargo_content.contains("ic-wasi-polyfill"),
            "Scenario {} should not have WASI polyfill: {}",
            name,
            cargo_content
        );
        assert!(
            !lib_content.contains("wasi_init"),
            "Scenario {} should not have WASI init: {}",
            name,
            lib_content
        );

        // Consistent presence of core dependencies
        assert!(
            cargo_content.contains("ic-cdk"),
            "Scenario {} should have ic-cdk: {}",
            name,
            cargo_content
        );
    }
}

/// Test non-WASI project build configuration
#[test]
fn test_non_wasi_build_configuration() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-build-config");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-build-config"],
    );
    assert_success(&output);

    // Create cargo config for local dependencies
    project.create_cargo_config_override();

    // Test build with pure WASM flag (should be redundant but work)
    let build_output = runner.run_in(&project.project_dir(), &["build", "--pure-wasm"]);

    let stdout = String::from_utf8_lossy(&build_output.stdout);
    let stderr = String::from_utf8_lossy(&build_output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // Should confirm pure WASM usage
    if build_output.status.success() {
        assert!(
            combined_output.contains("pure WASM")
                || combined_output.contains("wasm32-unknown-unknown"),
            "Pure WASM flag should be confirmed: {}",
            combined_output
        );
    } else {
        // If build fails, it should be due to dependencies, not WASI conflicts
        assert!(
            !combined_output.contains("wasi") && !combined_output.contains("wasip1"),
            "Build failure should not be WASI-related: {}",
            combined_output
        );
    }
}

/// Test non-WASI project validation workflow
#[test]
fn test_non_wasi_project_validation() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-validation");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-validation"],
    );
    assert_success(&output);

    // Comprehensive non-WASI validation
    struct NonWasiValidation<'a> {
        cargo_content: &'a str,
        lib_content: &'a str,
    }

    impl<'a> NonWasiValidation<'a> {
        fn validate_no_wasi_dependencies(&self) -> bool {
            !self.cargo_content.contains("ic-wasi-polyfill")
        }

        fn validate_no_wasi_features(&self) -> bool {
            !self.cargo_content.contains("wasi = [")
                && !self.cargo_content.contains("default = [\"wasi\"]")
        }

        fn validate_no_wasi_initialization(&self) -> bool {
            !self.lib_content.contains("wasi_init")
        }

        fn validate_core_dependencies(&self) -> bool {
            self.cargo_content.contains("ic-cdk")
                && self.cargo_content.contains("candid")
                && self.cargo_content.contains("serde")
        }

        fn validate_optimization_settings(&self) -> bool {
            self.cargo_content.contains("lto = true")
                && self.cargo_content.contains("strip = \"debuginfo\"")
        }

        fn validate_all(&self) -> bool {
            self.validate_no_wasi_dependencies()
                && self.validate_no_wasi_features()
                && self.validate_no_wasi_initialization()
                && self.validate_core_dependencies()
                && self.validate_optimization_settings()
        }
    }

    let cargo_content = project.read_file("Cargo.toml");
    let lib_content = project.read_file("src/lib.rs");

    let validation = NonWasiValidation {
        cargo_content: &cargo_content,
        lib_content: &lib_content,
    };

    assert!(
        validation.validate_no_wasi_dependencies(),
        "Should not have WASI dependencies"
    );
    assert!(
        validation.validate_no_wasi_features(),
        "Should not have WASI features"
    );
    assert!(
        validation.validate_no_wasi_initialization(),
        "Should not have WASI initialization"
    );
    assert!(
        validation.validate_core_dependencies(),
        "Should have core IC dependencies"
    );
    assert!(
        validation.validate_optimization_settings(),
        "Should have proper optimization settings"
    );
    assert!(
        validation.validate_all(),
        "Complete non-WASI validation should pass"
    );
}

/// Test non-WASI project performance characteristics
#[test]
fn test_non_wasi_project_performance() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-non-wasi-performance");

    // Create non-WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-non-wasi-performance"],
    );
    assert_success(&output);

    // Verify performance-oriented configuration
    let cargo_content = project.read_file("Cargo.toml");

    // Performance settings for pure WASM
    assert!(
        cargo_content.contains("panic = \"abort\""),
        "Should use abort on panic for smaller WASM: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("overflow-checks = false"),
        "Should disable overflow checks in release: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("debug = false"),
        "Should disable debug info in release: {}",
        cargo_content
    );

    // Should be optimized for simple, efficient canisters
    let lib_content = project.read_file("src/lib.rs");
    assert!(
        lib_content.len() < 2000, // Should be relatively minimal
        "lib.rs should be minimal for simple canisters: {} bytes",
        lib_content.len()
    );
}
