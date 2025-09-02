//! E2E test helper utilities

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// Helper for running CLI commands
pub struct CliRunner {
    binary_path: PathBuf,
}

impl CliRunner {
    /// Create a new CLI runner using the built binary
    pub fn new() -> Self {
        // The binary should already be built before running tests
        let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("target")
            .join("release")
            .join("icarus");

        // If not built, build it now
        if !binary_path.exists() {
            let output = Command::new("cargo")
                .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
                .args(&[
                    "build",
                    "--package",
                    "icarus-cli",
                    "--bin",
                    "icarus",
                    "--release",
                ])
                .output()
                .expect("Failed to build icarus CLI");

            assert!(output.status.success(), "Failed to build CLI: {:?}", output);
        }

        assert!(
            binary_path.exists(),
            "CLI binary not found at {:?}",
            binary_path
        );

        Self { binary_path }
    }

    /// Run a command with arguments
    #[allow(dead_code)]
    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(&self.binary_path)
            .args(args)
            .output()
            .expect("Failed to execute CLI command")
    }

    /// Run a command in a specific directory
    pub fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(&self.binary_path)
            .current_dir(dir)
            .args(args)
            .output()
            .expect("Failed to execute CLI command")
    }
}

/// Helper for managing test projects
pub struct TestProject {
    dir: TempDir,
    name: String,
}

impl TestProject {
    /// Create a new test project directory
    pub fn new(name: &str) -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir");
        Self {
            dir,
            name: name.to_string(),
        }
    }

    /// Get the project path
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Get the project directory path
    pub fn project_dir(&self) -> PathBuf {
        self.dir.path().join(&self.name)
    }

    /// Check if a file exists in the project
    #[allow(dead_code)]
    pub fn file_exists(&self, path: &str) -> bool {
        self.project_dir().join(path).exists()
    }

    /// Read a file from the project
    pub fn read_file(&self, path: &str) -> String {
        fs::read_to_string(self.project_dir().join(path))
            .expect(&format!("Failed to read file: {}", path))
    }

    /// Write a file to the project
    #[allow(dead_code)]
    pub fn write_file(&self, path: &str, content: &str) {
        let file_path = self.project_dir().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
        fs::write(file_path, content).expect(&format!("Failed to write file: {}", path));
    }

    /// Run cargo build in the project
    #[allow(dead_code)]
    pub fn cargo_build(&self) -> Output {
        Command::new("cargo")
            .current_dir(self.project_dir())
            .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
            .output()
            .expect("Failed to run cargo build")
    }

    /// Check if the project builds successfully
    #[allow(dead_code)]
    pub fn builds_successfully(&self) -> bool {
        self.cargo_build().status.success()
    }
}

/// Helper for cleaning up test artifacts
#[allow(dead_code)]
pub struct TestCleanup;

impl TestCleanup {
    /// Run the project's clean script
    #[allow(dead_code)]
    pub fn run_clean_script() {
        let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("scripts")
            .join("clean.sh");

        if script_path.exists() {
            Command::new("bash")
                .arg(script_path)
                .arg("--non-interactive")
                .output()
                .expect("Failed to run clean script");
        }
    }
}

/// Assert that command output contains a string
pub fn assert_contains(output: &Output, expected: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains(expected) || stderr.contains(expected),
        "Expected output to contain '{}'\nStdout: {}\nStderr: {}",
        expected,
        stdout,
        stderr
    );
}

/// Assert that command succeeded
pub fn assert_success(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Command failed with status: {:?}\nStdout: {}\nStderr: {}",
        output.status,
        stdout,
        stderr
    );
}
