//! E2E test helper utilities

// Allow dead code in test utilities - these are helper functions that may not all be used yet
#![allow(dead_code)]

pub mod parallel;

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
            let mut cmd = Command::new("cargo");
            // Clear coverage-related environment variables that break builds
            cmd.env_remove("LLVM_PROFILE_FILE")
                .env_remove("RUSTFLAGS")
                .env_remove("CARGO_INCREMENTAL")
                .env_remove("CARGO_LLVM_COV")
                .env_remove("CARGO_LLVM_COV_SHOW_ENV")
                .env_remove("CARGO_LLVM_COV_TARGET_DIR");

            let output = cmd
                .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
                .args([
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
    pub fn run(&self, args: &[&str]) -> Output {
        eprintln!("Running command: {:?} {:?}", self.binary_path, args);
        let output = Command::new(&self.binary_path)
            .args(args)
            .output()
            .expect("Failed to execute CLI command");
        eprintln!("Command completed with status: {}", output.status);
        output
    }

    /// Run a command in a specific directory
    pub fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        eprintln!(
            "Running command in {:?}: {:?} {:?}",
            dir, self.binary_path, args
        );
        let output = Command::new(&self.binary_path)
            .current_dir(dir)
            .args(args)
            .output()
            .expect("Failed to execute CLI command");
        eprintln!("Command completed with status: {}", output.status);
        output
    }

    /// Run a command in a specific directory with environment variables
    pub fn run_in_with_env(&self, dir: &Path, args: &[&str], env_vars: &[(&str, &str)]) -> Output {
        eprintln!(
            "Running command in {:?} with env {:?}: {:?} {:?}",
            dir, env_vars, self.binary_path, args
        );
        let mut cmd = Command::new(&self.binary_path);
        cmd.current_dir(dir).args(args);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output().expect("Failed to execute CLI command");
        eprintln!("Command completed with status: {}", output.status);
        output
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
    pub fn file_exists(&self, path: &str) -> bool {
        self.project_dir().join(path).exists()
    }

    /// Read a file from the project
    pub fn read_file(&self, path: &str) -> String {
        fs::read_to_string(self.project_dir().join(path))
            .unwrap_or_else(|_| panic!("Failed to read file: {}", path))
    }

    /// Write a file to the project
    pub fn write_file(&self, path: &str, content: &str) {
        let file_path = self.project_dir().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
        fs::write(file_path, content).unwrap_or_else(|_| panic!("Failed to write file: {}", path));
    }

    /// Create Cargo config override for testing with local workspace dependencies
    pub fn create_cargo_config_override(&self) {
        create_cargo_config_override(&self.project_dir());
    }

    /// Run cargo build in the project
    pub fn cargo_build(&self) -> Output {
        let mut cmd = Command::new("cargo");
        // Clear coverage-related environment variables that break WASM builds
        cmd.env_remove("LLVM_PROFILE_FILE")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_INCREMENTAL")
            .env_remove("CARGO_LLVM_COV")
            .env_remove("CARGO_LLVM_COV_SHOW_ENV")
            .env_remove("CARGO_LLVM_COV_TARGET_DIR");

        cmd.current_dir(self.project_dir())
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .output()
            .expect("Failed to run cargo build")
    }

    /// Check if the project builds successfully
    pub fn builds_successfully(&self) -> bool {
        self.cargo_build().status.success()
    }
}

/// Helper for cleaning up test artifacts
pub struct TestCleanup;

impl TestCleanup {
    /// Run the project's clean script
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

/// Create a .cargo/config.toml to use local workspace dependencies instead of crates.io
fn create_cargo_config_override(project_dir: &Path) {
    // Get the CDK root directory (parent of cli directory)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cdk_root = manifest_dir
        .parent()
        .expect("Failed to get CDK root directory")
        .to_path_buf();

    // Create .cargo directory
    let cargo_dir = project_dir.join(".cargo");
    fs::create_dir_all(&cargo_dir).expect("Failed to create .cargo directory");

    // Write config.toml with path overrides for all icarus crates
    let config_content = format!(
        r#"[patch.crates-io]
icarus = {{ path = "{}" }}
icarus-core = {{ path = "{}/crates/icarus-core" }}
icarus-derive = {{ path = "{}/crates/icarus-derive" }}
icarus-canister = {{ path = "{}/crates/icarus-canister" }}
"#,
        cdk_root.display(),
        cdk_root.display(),
        cdk_root.display(),
        cdk_root.display()
    );

    let config_path = cargo_dir.join("config.toml");
    fs::write(&config_path, config_content).expect("Failed to write .cargo/config.toml");

    println!(
        "Created Cargo config override at: {}",
        config_path.display()
    );
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            // Skip target directory to avoid copying build artifacts
            if entry.file_name() != "target" {
                copy_dir_all(&src_path, &dst_path)?;
            }
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
