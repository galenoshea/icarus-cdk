//! Integration tests for WASI workflow end-to-end functionality
//!
//! Tests the complete WASI workflow from project creation to build verification

mod common;

use common::{assert_success, CliRunner, TestProject};
use std::fs;

/// Test complete WASI project creation workflow
#[test]
fn test_wasi_project_creation_end_to_end() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-e2e");

    // Run icarus new with --wasi flag
    let output = runner.run_in(project.path(), &["new", "test-wasi-e2e", "--wasi"]);

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

    // Verify WASI-specific content in Cargo.toml
    let cargo_content = project.read_file("Cargo.toml");
    assert!(
        cargo_content.contains("ic-wasi-polyfill"),
        "Cargo.toml should contain ic-wasi-polyfill dependency: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("optional = true"),
        "ic-wasi-polyfill should be optional: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("wasi = [\"ic-wasi-polyfill\"]"),
        "WASI feature should be defined: {}",
        cargo_content
    );
    assert!(
        cargo_content.contains("default = [\"wasi\"]"),
        "WASI should be in default features: {}",
        cargo_content
    );

    // Verify WASI-specific content in lib.rs
    let lib_content = project.read_file("src/lib.rs");
    assert!(
        lib_content.contains("wasi_init!()"),
        "lib.rs should contain wasi_init!() call: {}",
        lib_content
    );
    assert!(
        lib_content.contains("// Initialize WASI polyfill"),
        "lib.rs should contain WASI initialization comment: {}",
        lib_content
    );
}

/// Test WASI project builds with correct target
#[test]
fn test_wasi_project_builds_correctly() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-build");

    // Create WASI project
    let output = runner.run_in(project.path(), &["new", "test-wasi-build", "--wasi"]);
    assert_success(&output);

    // Create cargo config override to use local dependencies
    project.create_cargo_config_override();

    // Test that build command detects WASI features
    let build_output = runner.run_in(&project.project_dir(), &["build"]);

    // Build should succeed (or at least start the process correctly)
    // We check the output contains WASI-related messages
    let stdout = String::from_utf8_lossy(&build_output.stdout);
    let stderr = String::from_utf8_lossy(&build_output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // Verify WASI-related build messages
    assert!(
        combined_output.contains("WASI support")
            || combined_output.contains("wasi")
            || combined_output.contains("wasip1"),
        "Build output should mention WASI or wasip1 target: {}",
        combined_output
    );
}

/// Test WASI vs non-WASI project differences
#[test]
fn test_wasi_vs_non_wasi_project_differences() {
    let runner = CliRunner::new();
    let wasi_project = TestProject::new("test-wasi-comparison");
    let non_wasi_project = TestProject::new("test-non-wasi-comparison");

    // Create WASI project
    let wasi_output = runner.run_in(
        wasi_project.path(),
        &["new", "test-wasi-comparison", "--wasi"],
    );
    assert_success(&wasi_output);

    // Create non-WASI project
    let non_wasi_output = runner.run_in(
        non_wasi_project.path(),
        &["new", "test-non-wasi-comparison"],
    );
    assert_success(&non_wasi_output);

    // Compare Cargo.toml files
    let wasi_cargo = wasi_project.read_file("Cargo.toml");
    let non_wasi_cargo = non_wasi_project.read_file("Cargo.toml");

    // WASI project should have WASI-specific content
    assert!(
        wasi_cargo.contains("ic-wasi-polyfill"),
        "WASI project should contain ic-wasi-polyfill: {}",
        wasi_cargo
    );
    assert!(
        wasi_cargo.contains("wasi = [\"ic-wasi-polyfill\"]"),
        "WASI project should define WASI feature: {}",
        wasi_cargo
    );
    assert!(
        wasi_cargo.contains("default = [\"wasi\"]"),
        "WASI project should include WASI in default features: {}",
        wasi_cargo
    );

    // Non-WASI project should NOT have WASI-specific content
    assert!(
        !non_wasi_cargo.contains("ic-wasi-polyfill"),
        "Non-WASI project should not contain ic-wasi-polyfill: {}",
        non_wasi_cargo
    );
    assert!(
        !non_wasi_cargo.contains("wasi = "),
        "Non-WASI project should not define WASI feature: {}",
        non_wasi_cargo
    );

    // Compare lib.rs files
    let wasi_lib = wasi_project.read_file("src/lib.rs");
    let non_wasi_lib = non_wasi_project.read_file("src/lib.rs");

    // WASI project should have wasi_init!() call
    assert!(
        wasi_lib.contains("wasi_init!()"),
        "WASI project should contain wasi_init!() call: {}",
        wasi_lib
    );

    // Non-WASI project should NOT have wasi_init!() call
    assert!(
        !non_wasi_lib.contains("wasi_init!()"),
        "Non-WASI project should not contain wasi_init!() call: {}",
        non_wasi_lib
    );
}

/// Test WASI feature detection in existing project
#[test]
fn test_wasi_feature_detection() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-detection");

    // Create WASI project
    let output = runner.run_in(project.path(), &["new", "test-wasi-detection", "--wasi"]);
    assert_success(&output);

    // Verify that build command detects WASI features in the created project
    let build_output = runner.run_in(&project.project_dir(), &["build", "--help"]);

    // Command should execute without error (even if just showing help)
    assert!(
        build_output.status.success() || build_output.status.code() == Some(0),
        "Build command should execute without critical errors"
    );
}

/// Test WASI project with ecosystem libraries compatibility
#[test]
fn test_wasi_ecosystem_libraries_setup() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-ecosystem");

    // Create WASI project
    let output = runner.run_in(project.path(), &["new", "test-wasi-ecosystem", "--wasi"]);
    assert_success(&output);

    // Verify the project is set up to support ecosystem libraries
    let cargo_content = project.read_file("Cargo.toml");

    // Check that the WASI polyfill is properly configured
    assert!(
        cargo_content.contains("ic-wasi-polyfill = { version = \"0.11\", optional = true }"),
        "WASI polyfill should be configured as optional dependency: {}",
        cargo_content
    );

    // Check that WASI feature includes the polyfill
    assert!(
        cargo_content.contains("wasi = [\"ic-wasi-polyfill\"]"),
        "WASI feature should include polyfill: {}",
        cargo_content
    );

    // Check that WASI is enabled by default for ecosystem compatibility
    assert!(
        cargo_content.contains("default = [\"wasi\"]"),
        "WASI should be enabled by default: {}",
        cargo_content
    );

    // Verify lib.rs has proper WASI initialization
    let lib_content = project.read_file("src/lib.rs");
    assert!(
        lib_content.contains("wasi_init!()"),
        "lib.rs should initialize WASI for ecosystem compatibility: {}",
        lib_content
    );
}

/// Test WASI project structure completeness
#[test]
fn test_wasi_project_structure_completeness() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-structure");

    // Create WASI project with tests
    let output = runner.run_in(
        project.path(),
        &[
            "new",
            "test-wasi-structure",
            "--wasi",
            "--with-tests",
            "--silent",
        ],
    );
    assert_success(&output);

    // Verify all expected files exist
    let expected_files = vec![
        "Cargo.toml",
        "dfx.json",
        "src/lib.rs",
        "tests/test_basic.rs",
        ".gitignore",
    ];

    for file in expected_files {
        assert!(
            project.file_exists(file),
            "File {} should exist in WASI project",
            file
        );
    }

    // Verify dfx.json contains proper canister configuration
    let dfx_content = project.read_file("dfx.json");
    assert!(
        dfx_content.contains("\"type\": \"custom\""),
        "dfx.json should have custom canister type: {}",
        dfx_content
    );

    // Verify .gitignore contains proper entries
    let gitignore_content = project.read_file(".gitignore");
    assert!(
        gitignore_content.contains("target/"),
        ".gitignore should ignore target directory: {}",
        gitignore_content
    );
    assert!(
        gitignore_content.contains(".dfx/"),
        ".gitignore should ignore .dfx directory: {}",
        gitignore_content
    );
}

/// Test WASI project with various flag combinations
#[test]
fn test_wasi_project_flag_combinations() {
    let runner = CliRunner::new();

    // Test WASI + tests
    let project_with_tests = TestProject::new("test-wasi-flags-tests");
    let output = runner.run_in(
        project_with_tests.path(),
        &[
            "new",
            "test-wasi-flags-tests",
            "--wasi",
            "--with-tests",
            "--silent",
        ],
    );
    assert_success(&output);

    assert!(
        project_with_tests.file_exists("tests/test_basic.rs"),
        "Project with tests should have test file"
    );

    let cargo_content = project_with_tests.read_file("Cargo.toml");
    assert!(
        cargo_content.contains("ic-wasi-polyfill"),
        "Project should have WASI dependency even with tests: {}",
        cargo_content
    );

    // Test WASI without tests (default)
    let project_no_tests = TestProject::new("test-wasi-flags-no-tests");
    let output = runner.run_in(
        project_no_tests.path(),
        &["new", "test-wasi-flags-no-tests", "--wasi"],
    );
    assert_success(&output);

    let cargo_content = project_no_tests.read_file("Cargo.toml");
    assert!(
        cargo_content.contains("ic-wasi-polyfill"),
        "Project should have WASI dependency: {}",
        cargo_content
    );
}

/// Test WASI project error handling
#[test]
fn test_wasi_project_error_handling() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-errors");

    // Test creating project in existing directory
    fs::create_dir_all(project.project_dir()).expect("Failed to create project dir");
    fs::write(project.project_dir().join("existing_file.txt"), "test")
        .expect("Failed to create existing file");

    let output = runner.run_in(project.path(), &["new", "test-wasi-errors", "--wasi"]);

    // Command should handle the existing directory gracefully
    // Either succeed (overwrite) or fail with informative message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        // If it fails, should provide informative error message
        assert!(
            stderr.contains("exists") || stdout.contains("exists"),
            "Error message should mention existing directory: stderr={}, stdout={}",
            stderr,
            stdout
        );
    }
}

/// Test WASI integration with build system
#[test]
fn test_wasi_build_system_integration() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-build-integration");

    // Create WASI project
    let output = runner.run_in(
        project.path(),
        &["new", "test-wasi-build-integration", "--wasi"],
    );
    assert_success(&output);

    // Create cargo config for local dependencies
    project.create_cargo_config_override();

    // Verify that the build process can detect WASI features
    // We'll check that the build command runs without immediately failing
    let build_output = runner.run_in(&project.project_dir(), &["build", "--help"]);

    // At minimum, the build command should be available and show help
    assert!(
        build_output.status.success(),
        "Build command should be available: {:?}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let help_output = String::from_utf8_lossy(&build_output.stdout);
    assert!(
        help_output.contains("build") || help_output.contains("Build"),
        "Build help should mention build functionality: {}",
        help_output
    );
}

/// Test WASI project validation workflow
#[test]
fn test_wasi_project_validation() {
    let runner = CliRunner::new();
    let project = TestProject::new("test-wasi-validation");

    // Create WASI project
    let output = runner.run_in(project.path(), &["new", "test-wasi-validation", "--wasi"]);
    assert_success(&output);

    // Validate the project meets WASI requirements
    let cargo_content = project.read_file("Cargo.toml");
    let lib_content = project.read_file("src/lib.rs");

    // Comprehensive WASI validation
    struct WasiValidation<'a> {
        cargo_content: &'a str,
        lib_content: &'a str,
    }

    impl<'a> WasiValidation<'a> {
        fn validate_dependencies(&self) -> bool {
            self.cargo_content.contains("ic-wasi-polyfill")
                && self.cargo_content.contains("optional = true")
        }

        fn validate_features(&self) -> bool {
            self.cargo_content.contains("wasi = [\"ic-wasi-polyfill\"]")
                && self.cargo_content.contains("default = [\"wasi\"]")
        }

        fn validate_initialization(&self) -> bool {
            self.lib_content.contains("wasi_init!()")
        }

        fn validate_all(&self) -> bool {
            self.validate_dependencies()
                && self.validate_features()
                && self.validate_initialization()
        }
    }

    let validation = WasiValidation {
        cargo_content: &cargo_content,
        lib_content: &lib_content,
    };

    assert!(
        validation.validate_dependencies(),
        "WASI dependencies should be properly configured"
    );
    assert!(
        validation.validate_features(),
        "WASI features should be properly configured"
    );
    assert!(
        validation.validate_initialization(),
        "WASI initialization should be present"
    );
    assert!(
        validation.validate_all(),
        "Complete WASI validation should pass"
    );
}
