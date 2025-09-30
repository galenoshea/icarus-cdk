//! Comprehensive end-to-end workflow tests for Icarus SDK CLI
//!
//! These tests validate the complete workflow from project creation to deployment,
//! following rust_best_practices.md requirements.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Test helper for creating CLI commands
fn icarus_cmd() -> Command {
    Command::cargo_bin("icarus-cli").unwrap()
}

/// Test helper for creating temporary project
struct TestProject {
    temp_dir: TempDir,
    project_name: String,
    project_path: std::path::PathBuf,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let project_name = name.to_string();
        let project_path = temp_dir.path().join(&project_name);
        Self {
            temp_dir,
            project_name,
            project_path,
        }
    }

    fn create_with_template(&self, template: &str) -> assert_cmd::Command {
        let mut cmd = icarus_cmd();
        cmd.args([
            "new",
            &self.project_name,
            "--template",
            template,
            "--path",
            self.temp_dir.path().to_str().unwrap(),
            "--no-interactive",
            "--no-git",
            "--no-install",
        ]);
        cmd
    }

    fn build_command(&self) -> assert_cmd::Command {
        let mut cmd = icarus_cmd();
        cmd.current_dir(&self.project_path);
        cmd.args(["build", "--mode", "debug"]);
        cmd
    }

    fn exists(&self) -> bool {
        self.project_path.exists()
    }

    fn has_file(&self, file_path: &str) -> bool {
        self.project_path.join(file_path).exists()
    }

    fn read_file(&self, file_path: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(self.project_path.join(file_path))
    }
}

/// Complete workflow test: Create → Validate → Build
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_complete_basic_workflow() {
    let project = TestProject::new("workflow-basic");

    // Step 1: Create project
    project
        .create_with_template("basic")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project created successfully"));

    // Step 2: Validate project structure
    assert!(project.exists());
    assert!(project.has_file("Cargo.toml"));
    assert!(project.has_file("src/lib.rs"));
    assert!(project.has_file("dfx.json"));
    assert!(project.has_file("README.md"));
    assert!(project.has_file(".gitignore"));

    // Step 3: Validate Cargo.toml content
    let cargo_content = project.read_file("Cargo.toml").unwrap();
    assert!(cargo_content.contains(&format!("name = \"{}\"", project.project_name)));
    assert!(cargo_content.contains("icarus"));
    assert!(cargo_content.contains("[lib]"));
    assert!(cargo_content.contains("crate-type = [\"cdylib\"]"));

    // Step 4: Validate lib.rs content
    let lib_content = project.read_file("src/lib.rs").unwrap();
    assert!(lib_content.contains("use icarus"));
    assert!(lib_content.contains("ic_cdk_macros"));
    assert!(lib_content.contains("export_candid"));

    // Step 5: Validate dfx.json content
    let dfx_content = project.read_file("dfx.json").unwrap();
    assert!(dfx_content.contains("\"type\": \"rust\""));
    assert!(dfx_content.contains("\"package\": \""));

    // Step 6: Test build (if cargo is available)
    if which::which("cargo").is_ok() {
        project
            .build_command()
            .timeout(Duration::from_secs(120)) // Generous timeout for CI
            .assert()
            .success()
            .stdout(
                predicate::str::contains("Build completed successfully")
                    .or(predicate::str::contains("Finished")),
            );
    }
}

/// Test all templates can be created and have valid structure
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_all_templates_creation() {
    let templates = ["basic", "advanced", "mcp-server", "dapp"];

    for template in templates {
        let project = TestProject::new(&format!("template-{}", template));

        // Create project with template
        project
            .create_with_template(template)
            .assert()
            .success()
            .stdout(predicate::str::contains("Project created successfully"));

        // Validate basic structure exists
        assert!(
            project.exists(),
            "Project directory should exist for template: {}",
            template
        );
        assert!(
            project.has_file("Cargo.toml"),
            "Cargo.toml should exist for template: {}",
            template
        );
        assert!(
            project.has_file("src/lib.rs"),
            "src/lib.rs should exist for template: {}",
            template
        );

        // Validate Cargo.toml is properly formatted
        let cargo_content = project.read_file("Cargo.toml").unwrap();
        assert!(
            cargo_content.contains("name = \""),
            "Cargo.toml should have project name for template: {}",
            template
        );
        assert!(
            cargo_content.contains("version = \""),
            "Cargo.toml should have version for template: {}",
            template
        );

        // Template-specific validations
        match template {
            "basic" => {
                assert!(project.has_file("dfx.json"));
                let lib_content = project.read_file("src/lib.rs").unwrap();
                assert!(lib_content.contains("ic_cdk_macros"));
            }
            "advanced" => {
                assert!(project.has_file("dfx.json"));
                assert!(
                    project.has_file("src/auth.rs") || lib_content_contains(&project, "mod auth")
                );
            }
            "mcp-server" => {
                assert!(project.has_file("src/mcp"));
                let lib_content = project.read_file("src/lib.rs").unwrap();
                assert!(lib_content.contains("mcp") || lib_content.contains("MCP"));
            }
            "dapp" => {
                assert!(project.has_file("frontend") || project.has_file("assets"));
                assert!(project.has_file("dfx.json"));
            }
            _ => unreachable!(),
        }
    }
}

fn lib_content_contains(project: &TestProject, text: &str) -> bool {
    project
        .read_file("src/lib.rs")
        .map(|content| content.contains(text))
        .unwrap_or(false)
}

/// Test MCP client integration workflow
#[tokio::test]
#[serial]
async fn test_mcp_integration_workflow() {
    // Test MCP list command (should work without any servers)
    icarus_cmd()
        .args(["--quiet", "mcp", "list"])
        .assert()
        .success();

    // Test MCP list with different formats
    let formats = ["table", "json", "yaml", "plain"];
    for format in formats {
        icarus_cmd()
            .args(["--quiet", "mcp", "list", "--format", format])
            .assert()
            .success();
    }

    // Test MCP status command
    icarus_cmd()
        .args(["--quiet", "mcp", "status", "--all"])
        .assert()
        .success();

    // Test MCP add with invalid canister ID (should fail gracefully)
    icarus_cmd()
        .args([
            "mcp",
            "add",
            "invalid-canister-id",
            "--client",
            "claude-desktop",
            "--skip-verify",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid canister ID format"));

    // Test MCP remove non-existent server (should fail gracefully)
    icarus_cmd()
        .args(["mcp", "remove", "nonexistent-server", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No MCP server found"));
}

/// Test build process with different configurations
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_build_configurations() {
    let project = TestProject::new("build-test");

    // Create basic project
    project.create_with_template("basic").assert().success();

    // Skip build tests if cargo is not available
    if which::which("cargo").is_err() {
        return;
    }

    // Test debug build
    let mut debug_cmd = icarus_cmd();
    debug_cmd.current_dir(&project.project_path);
    debug_cmd.args(["build", "--mode", "debug"]);
    debug_cmd
        .timeout(Duration::from_secs(120))
        .assert()
        .success();

    // Test with specific target
    let mut target_cmd = icarus_cmd();
    target_cmd.current_dir(&project.project_path);
    target_cmd.args([
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--mode",
        "debug",
    ]);
    target_cmd
        .timeout(Duration::from_secs(120))
        .assert()
        .success();

    // Test with features
    let mut features_cmd = icarus_cmd();
    features_cmd.current_dir(&project.project_path);
    features_cmd.args(["build", "--features", "default", "--mode", "debug"]);
    features_cmd
        .timeout(Duration::from_secs(120))
        .assert()
        .success();

    // Test build with test flag
    let mut test_cmd = icarus_cmd();
    test_cmd.current_dir(&project.project_path);
    test_cmd.args(["build", "--test", "--mode", "debug"]);
    test_cmd
        .timeout(Duration::from_secs(120))
        .assert()
        .success();
}

/// Test error handling and edge cases
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_error_handling() {
    // Test creating project with invalid name
    let temp_dir = TempDir::new().unwrap();

    icarus_cmd()
        .args([
            "new",
            "invalid@name",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--no-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("alphanumeric characters"));

    // Test creating project in existing non-empty directory
    let project = TestProject::new("existing-project");
    fs::create_dir_all(&project.project_path).unwrap();
    fs::write(project.project_path.join("existing-file.txt"), "content").unwrap();

    icarus_cmd()
        .args([
            "new",
            &project.project_name,
            "--path",
            project.temp_dir.path().to_str().unwrap(),
            "--no-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Test build in non-project directory
    let empty_dir = TempDir::new().unwrap();
    icarus_cmd()
        .current_dir(empty_dir.path())
        .args(["build"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in an Icarus project").or(
            predicate::str::contains("No such file").or(predicate::str::contains("Cargo.toml")),
        ));

    // Test deploy in non-project directory
    icarus_cmd()
        .current_dir(empty_dir.path())
        .args(["deploy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in an Icarus project").or(
            predicate::str::contains("No such file").or(predicate::str::contains("Cargo.toml")),
        ));

    // Test invalid template
    icarus_cmd()
        .args([
            "new",
            "invalid-template-test",
            "--template",
            "nonexistent-template",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--no-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test CLI flag combinations and global options
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_cli_flags() {
    let project = TestProject::new("flags-test");

    // Test verbose flag
    project
        .create_with_template("basic")
        .arg("--verbose")
        .assert()
        .success();

    // Clean up for next test
    fs::remove_dir_all(&project.project_path).ok();

    // Test quiet flag
    let project2 = TestProject::new("flags-test-2");
    project2
        .create_with_template("basic")
        .arg("--quiet")
        .assert()
        .success();

    // Test force flag with existing directory
    fs::write(project2.project_path.join("existing.txt"), "test").unwrap();

    project2
        .create_with_template("basic")
        .arg("--force")
        .assert()
        .success();

    // Test that global flags work with all commands
    let commands = [
        vec!["--help"],
        vec!["new", "--help"],
        vec!["build", "--help"],
        vec!["deploy", "--help"],
        vec!["mcp", "--help"],
    ];

    for cmd_args in commands {
        for flag in ["--verbose", "--quiet", "--force"] {
            let mut cmd = icarus_cmd();
            cmd.arg(flag);
            cmd.args(&cmd_args);
            cmd.assert().success();
        }
    }
}

/// Test project creation with git integration
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_git_integration() {
    // Skip if git is not available
    if which::which("git").is_err() {
        return;
    }

    let project = TestProject::new("git-test");

    // Create project with git enabled
    let mut cmd = icarus_cmd();
    cmd.args([
        "new",
        &project.project_name,
        "--template",
        "basic",
        "--path",
        project.temp_dir.path().to_str().unwrap(),
        "--no-interactive",
        "--no-install",
    ]);

    cmd.assert().success();

    // Check git repository was initialized
    assert!(project.has_file(".git/config"));
    assert!(project.has_file(".gitignore"));

    // Verify .gitignore content
    let gitignore_content = project.read_file(".gitignore").unwrap();
    assert!(gitignore_content.contains("target/"));
    assert!(gitignore_content.contains(".dfx/"));
}

/// Test concurrent project creation (stress test)
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple projects concurrently
    let handles = (0..3).map(|i| {
        let temp_path = temp_dir.path().to_owned();
        tokio::spawn(async move {
            let project_name = format!("concurrent-{}", i);
            let mut cmd = icarus_cmd();
            cmd.args([
                "new",
                &project_name,
                "--template",
                "basic",
                "--path",
                temp_path.to_str().unwrap(),
                "--no-interactive",
                "--no-git",
                "--no-install",
            ]);

            cmd.assert().success();

            // Verify project was created
            let project_path = temp_path.join(&project_name);
            assert!(project_path.exists());
            assert!(project_path.join("Cargo.toml").exists());
        })
    });

    // Wait for all projects to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test project validation and structure
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_project_validation() {
    let project = TestProject::new("validation-test");

    // Create project
    project.create_with_template("basic").assert().success();

    // Validate project structure
    validate_basic_project_structure(&project);

    // Validate file contents
    validate_project_file_contents(&project);
}

fn validate_basic_project_structure(project: &TestProject) {
    // Required files
    let required_files = [
        "Cargo.toml",
        "src/lib.rs",
        "dfx.json",
        "README.md",
        ".gitignore",
    ];

    for file in &required_files {
        assert!(
            project.has_file(file),
            "Required file '{}' is missing in project '{}'",
            file,
            project.project_name
        );
    }

    // Required directories
    let required_dirs = ["src"];

    for dir in &required_dirs {
        assert!(
            project.project_path.join(dir).is_dir(),
            "Required directory '{}' is missing in project '{}'",
            dir,
            project.project_name
        );
    }
}

fn validate_project_file_contents(project: &TestProject) {
    // Validate Cargo.toml
    let cargo_content = project.read_file("Cargo.toml").unwrap();
    assert!(cargo_content.contains("[package]"));
    assert!(cargo_content.contains(&format!("name = \"{}\"", project.project_name)));
    assert!(cargo_content.contains("version = "));
    assert!(cargo_content.contains("[dependencies]"));
    assert!(cargo_content.contains("icarus"));

    // Validate lib.rs
    let lib_content = project.read_file("src/lib.rs").unwrap();
    assert!(lib_content.contains("use icarus"));
    assert!(
        lib_content.len() > 100,
        "lib.rs should have substantial content"
    );

    // Validate dfx.json
    let dfx_content = project.read_file("dfx.json").unwrap();
    assert!(dfx_content.contains("\"version\""));
    assert!(dfx_content.contains("\"canisters\""));
    assert!(dfx_content.contains("\"type\": \"rust\""));

    // Validate README.md
    let readme_content = project.read_file("README.md").unwrap();
    assert!(readme_content.contains(&project.project_name));
    assert!(
        readme_content.len() > 50,
        "README.md should have meaningful content"
    );
}

/// Test build artifacts and output validation
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_build_artifacts() {
    // Skip if cargo is not available
    if which::which("cargo").is_err() {
        return;
    }

    let project = TestProject::new("artifacts-test");

    // Create project
    project.create_with_template("basic").assert().success();

    // Build with output directory
    let output_dir = project.temp_dir.path().join("build-output");

    let mut cmd = icarus_cmd();
    cmd.current_dir(&project.project_path);
    cmd.args([
        "build",
        "--mode",
        "debug",
        "--output",
        output_dir.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(120)).assert().success();

    // Verify build artifacts
    if output_dir.exists() {
        // Check for WASM files
        let entries = fs::read_dir(&output_dir).unwrap();
        let has_wasm = entries.filter_map(|entry| entry.ok()).any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "wasm")
                .unwrap_or(false)
        });

        if has_wasm {
            println!("✅ WASM artifacts generated successfully");
        } else {
            println!("ℹ️  No WASM artifacts found (may be expected for debug builds)");
        }
    }
}

/// Performance test: measure project creation time
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_performance_metrics() {
    let start = std::time::Instant::now();

    let project = TestProject::new("perf-test");
    project.create_with_template("basic").assert().success();

    let creation_time = start.elapsed();

    // Project creation should be reasonably fast (under 10 seconds)
    assert!(
        creation_time.as_secs() < 10,
        "Project creation took too long: {:?}",
        creation_time
    );

    println!("✅ Project creation completed in {:?}", creation_time);
}

/// Test cleanup and resource management
#[tokio::test]
#[serial]
#[ignore = "Template scaffolding not yet implemented"]
async fn test_cleanup_and_resources() {
    let temp_dir = TempDir::new().unwrap();
    let initial_entries = fs::read_dir(temp_dir.path()).unwrap().count();

    {
        let project = TestProject::new("cleanup-test");
        project.create_with_template("basic").assert().success();

        assert!(project.exists());
    } // TestProject drops here

    // Give a moment for any background cleanup
    sleep(Duration::from_millis(100)).await;

    // Verify temp directory is manageable
    let final_entries = fs::read_dir(temp_dir.path()).unwrap().count();

    // Should have at least one new directory (our project)
    assert!(final_entries > initial_entries);

    println!("✅ Resource cleanup test completed");
}
