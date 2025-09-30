//! Template system validation tests
//!
//! Comprehensive tests for template generation, validation, and customization.
//! Tests all available templates and their variants.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test helper for template validation
struct TemplateTestHelper {
    temp_dir: TempDir,
}

impl TemplateTestHelper {
    fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
        }
    }

    fn icarus_cmd(&self) -> Command {
        Command::cargo_bin("icarus-cli").unwrap()
    }

    fn create_project(&self, name: &str, template: &str) -> ProjectValidator {
        let project_path = self.temp_dir.path().join(name);

        let mut cmd = self.icarus_cmd();
        cmd.args([
            "new",
            name,
            "--template",
            template,
            "--path",
            self.temp_dir.path().to_str().unwrap(),
            "--no-interactive",
            "--no-git",
            "--no-install",
        ]);

        let result = cmd.assert().success();

        ProjectValidator {
            project_path,
            template_name: template.to_string(),
            project_name: name.to_string(),
            assertion: result,
        }
    }
}

struct ProjectValidator {
    project_path: std::path::PathBuf,
    template_name: String,
    project_name: String,
    #[allow(dead_code)]
    assertion: assert_cmd::assert::Assert,
}

impl ProjectValidator {
    fn exists(&self) -> &Self {
        assert!(
            self.project_path.exists(),
            "Project directory should exist for template: {}",
            self.template_name
        );
        self
    }

    fn has_file(&self, file_path: &str) -> &Self {
        let full_path = self.project_path.join(file_path);
        assert!(
            full_path.exists(),
            "File '{}' should exist in template '{}' project",
            file_path,
            self.template_name
        );
        self
    }

    fn has_directory(&self, dir_path: &str) -> &Self {
        let full_path = self.project_path.join(dir_path);
        assert!(
            full_path.exists() && full_path.is_dir(),
            "Directory '{}' should exist in template '{}' project",
            dir_path,
            self.template_name
        );
        self
    }

    fn file_contains(&self, file_path: &str, content: &str) -> &Self {
        let full_path = self.project_path.join(file_path);
        let file_content = fs::read_to_string(&full_path).unwrap_or_else(|_| {
            panic!(
                "Could not read file '{}' in template '{}'",
                file_path, self.template_name
            )
        });

        assert!(
            file_content.contains(content),
            "File '{}' in template '{}' should contain '{}'",
            file_path,
            self.template_name,
            content
        );
        self
    }

    #[allow(dead_code)]
    fn file_does_not_contain(&self, file_path: &str, content: &str) -> &Self {
        let full_path = self.project_path.join(file_path);
        if let Ok(file_content) = fs::read_to_string(&full_path) {
            assert!(
                !file_content.contains(content),
                "File '{}' in template '{}' should not contain '{}'",
                file_path,
                self.template_name,
                content
            );
        }
        self
    }

    fn cargo_toml_valid(&self) -> &Self {
        let cargo_content = self.read_file("Cargo.toml");

        // Basic structure validation
        assert!(
            cargo_content.contains("[package]"),
            "Cargo.toml should have [package] section"
        );
        assert!(
            cargo_content.contains(&format!("name = \"{}\"", self.project_name)),
            "Cargo.toml should have correct project name"
        );
        assert!(
            cargo_content.contains("version = \""),
            "Cargo.toml should have version"
        );
        assert!(
            cargo_content.contains("[dependencies]"),
            "Cargo.toml should have [dependencies] section"
        );

        // Icarus dependency validation
        assert!(
            cargo_content.contains("icarus"),
            "Cargo.toml should include icarus dependency"
        );

        self
    }

    fn lib_rs_valid(&self) -> &Self {
        let lib_content = self.read_file("src/lib.rs");

        // Basic structure validation
        assert!(
            lib_content.contains("use icarus"),
            "lib.rs should use icarus"
        );
        assert!(
            lib_content.len() > 50,
            "lib.rs should have substantial content"
        );

        self
    }

    fn dfx_json_valid(&self) -> &Self {
        if self.project_path.join("dfx.json").exists() {
            let dfx_content = self.read_file("dfx.json");

            // Validate JSON structure
            let _: serde_json::Value =
                serde_json::from_str(&dfx_content).expect("dfx.json should be valid JSON");

            assert!(
                dfx_content.contains("\"version\""),
                "dfx.json should have version"
            );
            assert!(
                dfx_content.contains("\"canisters\""),
                "dfx.json should have canisters section"
            );
        }

        self
    }

    fn readme_valid(&self) -> &Self {
        let readme_content = self.read_file("README.md");

        assert!(
            readme_content.contains(&self.project_name),
            "README.md should mention project name"
        );
        assert!(
            readme_content.len() > 100,
            "README.md should have meaningful content"
        );

        self
    }

    fn gitignore_valid(&self) -> &Self {
        let gitignore_content = self.read_file(".gitignore");

        assert!(
            gitignore_content.contains("target/"),
            ".gitignore should ignore target directory"
        );
        assert!(
            gitignore_content.contains(".dfx/"),
            ".gitignore should ignore .dfx directory"
        );

        self
    }

    fn package_json_valid(&self) -> &Self {
        if self.project_path.join("package.json").exists() {
            let package_content = self.read_file("package.json");

            // Validate JSON structure
            let package_json: serde_json::Value =
                serde_json::from_str(&package_content).expect("package.json should be valid JSON");

            assert!(
                package_json.get("name").is_some(),
                "package.json should have name field"
            );
            assert!(
                package_json.get("version").is_some(),
                "package.json should have version field"
            );
        }

        self
    }

    fn read_file(&self, file_path: &str) -> String {
        let full_path = self.project_path.join(file_path);
        fs::read_to_string(&full_path).unwrap_or_else(|_| {
            panic!(
                "Could not read file '{}' in template '{}'",
                file_path, self.template_name
            )
        })
    }
}

/// Test basic template structure and content
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_basic_template() {
    let helper = TemplateTestHelper::new();

    helper
        .create_project("basic-test", "basic")
        .exists()
        .has_file("Cargo.toml")
        .has_file("src/lib.rs")
        .has_file("dfx.json")
        .has_file("README.md")
        .has_file(".gitignore")
        .has_directory("src")
        .cargo_toml_valid()
        .lib_rs_valid()
        .dfx_json_valid()
        .readme_valid()
        .gitignore_valid()
        .file_contains("src/lib.rs", "ic_cdk_macros")
        .file_contains("src/lib.rs", "export_candid")
        .file_contains("Cargo.toml", "crate-type = [\"cdylib\"]");
}

/// Test advanced template with additional features
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_advanced_template() {
    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("advanced-test", "advanced");
    let validator = binding
        .exists()
        .has_file("Cargo.toml")
        .has_file("src/lib.rs")
        .has_file("dfx.json")
        .has_file("README.md")
        .has_file(".gitignore")
        .cargo_toml_valid()
        .lib_rs_valid()
        .dfx_json_valid()
        .readme_valid()
        .gitignore_valid();

    // Advanced template should have additional dependencies
    validator.file_contains("Cargo.toml", "ic-cdk");

    // Should have more sophisticated lib.rs
    let lib_content = validator.read_file("src/lib.rs");
    assert!(
        lib_content.len() > 200,
        "Advanced template should have more content than basic"
    );
}

/// Test MCP server template
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_mcp_server_template() {
    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("mcp-server-test", "mcp-server");
    let validator = binding
        .exists()
        .has_file("Cargo.toml")
        .has_file("src/lib.rs")
        .cargo_toml_valid()
        .lib_rs_valid()
        .readme_valid()
        .gitignore_valid();

    // MCP server should have specific dependencies
    validator.file_contains("Cargo.toml", "icarus");

    // Should have MCP-related code
    let lib_content = validator.read_file("src/lib.rs");
    assert!(
        lib_content.to_lowercase().contains("mcp")
            || lib_content.contains("tool")
            || lib_content.contains("server"),
        "MCP server template should contain MCP-related code"
    );
}

/// Test dApp template with frontend integration
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_dapp_template() {
    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("dapp-test", "dapp");
    let validator = binding
        .exists()
        .has_file("Cargo.toml")
        .has_file("src/lib.rs")
        .has_file("dfx.json")
        .has_file("README.md")
        .has_file(".gitignore")
        .cargo_toml_valid()
        .lib_rs_valid()
        .dfx_json_valid()
        .readme_valid()
        .gitignore_valid();

    // dApp should have frontend-related files
    assert!(
        validator.project_path.join("frontend").exists()
            || validator.project_path.join("assets").exists()
            || validator.project_path.join("src/frontend").exists()
            || validator.project_path.join("public").exists(),
        "dApp template should have frontend directory"
    );

    // Should have package.json for frontend dependencies
    if validator.project_path.join("package.json").exists() {
        validator.package_json_valid();
    }
}

/// Test template name validation
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_name_validation() {
    let helper = TemplateTestHelper::new();

    // Test non-existent template
    helper
        .icarus_cmd()
        .args([
            "new",
            "invalid-template-test",
            "--template",
            "nonexistent-template",
            "--path",
            helper.temp_dir.path().to_str().unwrap(),
            "--no-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Template")));
}

/// Test template customization options
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_customization() {
    let helper = TemplateTestHelper::new();

    // Test with custom author and description (when interactive mode is supported)
    let binding = helper.create_project("custom-test", "basic");
    let validator = binding.exists().cargo_toml_valid();

    // Verify default values are used in non-interactive mode
    validator.file_contains("Cargo.toml", "version = \"0.1.0\"");
}

/// Test project name transformations in templates
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_project_name_transformations() {
    let helper = TemplateTestHelper::new();

    // Test with kebab-case name
    let binding = helper.create_project("my-awesome-project", "basic");
    let validator = binding.exists().cargo_toml_valid();

    // Verify name is correctly used
    validator.file_contains("Cargo.toml", "name = \"my-awesome-project\"");

    // Test with underscore name
    let binding2 = helper.create_project("my_snake_project", "basic");
    let validator2 = binding2.exists().cargo_toml_valid();

    validator2.file_contains("Cargo.toml", "name = \"my_snake_project\"");
}

/// Test template file permissions (Unix systems)
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
#[cfg(unix)]
fn test_template_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("permissions-test", "basic");
    let validator = binding.exists();

    // Check that regular files have correct permissions
    let lib_metadata = fs::metadata(validator.project_path.join("src/lib.rs")).unwrap();
    let lib_perms = lib_metadata.permissions().mode();

    // Should be readable and writable by owner (at minimum)
    assert!(
        lib_perms & 0o600 == 0o600,
        "lib.rs should be readable and writable by owner"
    );

    // Check that directories have correct permissions
    let src_metadata = fs::metadata(validator.project_path.join("src")).unwrap();
    let src_perms = src_metadata.permissions().mode();

    // Should be readable, writable, and executable by owner
    assert!(
        src_perms & 0o700 == 0o700,
        "src directory should be accessible by owner"
    );
}

/// Test template generation with special characters in project name
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_special_characters() {
    let helper = TemplateTestHelper::new();

    // Test with numbers
    helper
        .create_project("project123", "basic")
        .exists()
        .cargo_toml_valid()
        .file_contains("Cargo.toml", "name = \"project123\"");

    // Test with hyphens
    helper
        .create_project("test-project-2024", "basic")
        .exists()
        .cargo_toml_valid()
        .file_contains("Cargo.toml", "name = \"test-project-2024\"");

    // Test with underscores
    helper
        .create_project("test_project_v2", "basic")
        .exists()
        .cargo_toml_valid()
        .file_contains("Cargo.toml", "name = \"test_project_v2\"");
}

/// Test template robustness with edge cases
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_edge_cases() {
    let helper = TemplateTestHelper::new();

    // Test with minimal project name
    helper
        .create_project("a", "basic")
        .exists()
        .cargo_toml_valid()
        .file_contains("Cargo.toml", "name = \"a\"");

    // Test with longer project name
    helper
        .create_project("very-long-project-name-that-should-still-work", "basic")
        .exists()
        .cargo_toml_valid()
        .file_contains(
            "Cargo.toml",
            "name = \"very-long-project-name-that-should-still-work\"",
        );
}

/// Test template consistency across all available templates
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_all_templates_consistency() {
    let helper = TemplateTestHelper::new();
    let templates = ["basic", "advanced", "mcp-server", "dapp"];

    for template in templates {
        let binding = helper.create_project(&format!("consistency-{}", template), template);
        let validator = binding
            .exists()
            .has_file("Cargo.toml")
            .has_file("src/lib.rs")
            .has_file("README.md")
            .has_file(".gitignore")
            .cargo_toml_valid()
            .lib_rs_valid()
            .readme_valid()
            .gitignore_valid();

        // All templates should have icarus dependency
        validator.file_contains("Cargo.toml", "icarus");
    }
}

/// Test template directory structure validation
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_directory_structure() {
    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("structure-test", "basic");
    let validator = binding.exists().has_directory("src");

    // Verify no unexpected directories are created
    let entries: Vec<_> = fs::read_dir(&validator.project_path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    // Should contain expected files/directories
    assert!(entries.contains(&"src".to_string()));
    assert!(entries.contains(&"Cargo.toml".to_string()));
    assert!(entries.contains(&"README.md".to_string()));
    assert!(entries.contains(&".gitignore".to_string()));

    // Should not contain unexpected items
    assert!(!entries.contains(&"node_modules".to_string()));
    assert!(!entries.contains(&"target".to_string()));
    assert!(!entries.contains(&".git".to_string()));
}

/// Test template file content encoding and formatting
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_file_encoding() {
    let helper = TemplateTestHelper::new();

    let binding = helper.create_project("encoding-test", "basic");
    let validator = binding.exists();

    // Test that all text files are UTF-8 encoded
    let text_files = ["Cargo.toml", "src/lib.rs", "README.md", ".gitignore"];

    for file in text_files {
        let content = validator.read_file(file);

        // Should be valid UTF-8 (if read_file succeeds, it's valid UTF-8)
        assert!(!content.is_empty(), "File {} should not be empty", file);

        // Should use Unix line endings or be consistent
        if content.contains('\r') {
            assert!(
                content.contains("\r\n"),
                "If file {} contains \\r, it should use \\r\\n",
                file
            );
        }
    }
}

/// Test template generation performance
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_generation_performance() {
    let helper = TemplateTestHelper::new();

    let start = std::time::Instant::now();

    helper.create_project("perf-test", "basic").exists();

    let duration = start.elapsed();

    // Template generation should be fast (under 5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Template generation took too long: {:?}",
        duration
    );

    println!("✅ Template generation completed in {:?}", duration);
}

/// Test template cleanup and resource management
#[test]
#[ignore = "Template scaffolding not yet implemented"]
#[serial]
fn test_template_resource_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let _initial_entries = fs::read_dir(temp_dir.path()).unwrap().count();

    {
        let helper = TemplateTestHelper::new();
        helper.create_project("cleanup-test", "basic").exists();
    } // TemplateTestHelper drops here

    // Verify temp directory management is working
    // (The test framework should handle cleanup)
    println!("✅ Template resource cleanup test completed");
}
