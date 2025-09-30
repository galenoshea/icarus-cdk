//! Test utilities and helpers for Icarus CLI tests
//! 
//! Provides common functionality and setup for integration tests.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test configuration and environment setup
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub config_dir: PathBuf,
    pub original_env: Vec<(String, Option<String>)>,
}

impl TestEnvironment {
    /// Create a new isolated test environment
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_dir = temp_dir.path().join(".icarus");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Capture original environment variables
        let original_env = vec![
            ("ICARUS_CONFIG_DIR".to_string(), std::env::var("ICARUS_CONFIG_DIR").ok()),
            ("CARGO_TARGET_DIR".to_string(), std::env::var("CARGO_TARGET_DIR").ok()),
            ("HOME".to_string(), std::env::var("HOME").ok()),
        ];

        Self {
            temp_dir,
            config_dir,
            original_env,
        }
    }

    /// Create an icarus command with environment setup
    pub fn icarus_cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("icarus-cli").expect("Failed to find icarus-cli binary");
        cmd.env("ICARUS_CONFIG_DIR", &self.config_dir);
        cmd.env("CARGO_TARGET_DIR", self.temp_dir.path().join("target"));
        cmd.env("HOME", self.temp_dir.path());
        cmd
    }

    /// Get the path to the MCP config file
    pub fn mcp_config_path(&self) -> PathBuf {
        self.config_dir.join("mcp_config.json")
    }

    /// Create a project directory within the test environment
    pub fn project_path(&self, name: &str) -> PathBuf {
        self.temp_dir.path().join(name)
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore original environment variables
        for (key, value) in &self.original_env {
            match value {
                Some(val) => std::env::set_var(key, val),
                None => std::env::remove_var(key),
            }
        }
    }
}

/// Project test helper with validation capabilities
pub struct ProjectTester {
    pub project_path: PathBuf,
    pub project_name: String,
    pub template_name: String,
}

impl ProjectTester {
    /// Create a new project tester
    pub fn new(env: &TestEnvironment, name: &str, template: &str) -> Self {
        let project_path = env.project_path(name);
        
        Self {
            project_path,
            project_name: name.to_string(),
            template_name: template.to_string(),
        }
    }

    /// Create the project using icarus new command
    pub fn create(&self, env: &TestEnvironment) -> assert_cmd::assert::Assert {
        env.icarus_cmd()
            .args([
                "new",
                &self.project_name,
                "--template",
                &self.template_name,
                "--path",
                env.temp_dir.path().to_str().unwrap(),
                "--no-interactive",
                "--no-git",
                "--no-install",
            ])
            .assert()
            .success()
    }

    /// Check if project exists
    pub fn exists(&self) -> bool {
        self.project_path.exists()
    }

    /// Check if a file exists in the project
    pub fn has_file(&self, file_path: &str) -> bool {
        self.project_path.join(file_path).exists()
    }

    /// Check if a directory exists in the project
    pub fn has_directory(&self, dir_path: &str) -> bool {
        let path = self.project_path.join(dir_path);
        path.exists() && path.is_dir()
    }

    /// Read a file from the project
    pub fn read_file(&self, file_path: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(self.project_path.join(file_path))
    }

    /// Build the project
    pub fn build(&self, env: &TestEnvironment) -> assert_cmd::assert::Assert {
        env.icarus_cmd()
            .current_dir(&self.project_path)
            .args(["build", "--mode", "debug"])
            .timeout(std::time::Duration::from_secs(120))
            .assert()
    }

    /// Run basic validation on the project structure
    pub fn validate_basic_structure(&self) -> Result<(), String> {
        let required_files = ["Cargo.toml", "src/lib.rs", "README.md", ".gitignore"];
        
        for file in &required_files {
            if !self.has_file(file) {
                return Err(format!("Missing required file: {}", file));
            }
        }

        if !self.has_directory("src") {
            return Err("Missing src directory".to_string());
        }

        Ok(())
    }

    /// Validate Cargo.toml content
    pub fn validate_cargo_toml(&self) -> Result<(), String> {
        let content = self.read_file("Cargo.toml")
            .map_err(|e| format!("Could not read Cargo.toml: {}", e))?;

        let required_sections = ["[package]", "[dependencies]"];
        for section in &required_sections {
            if !content.contains(section) {
                return Err(format!("Cargo.toml missing section: {}", section));
            }
        }

        if !content.contains(&format!("name = \"{}\"", self.project_name)) {
            return Err("Cargo.toml missing correct project name".to_string());
        }

        if !content.contains("icarus") {
            return Err("Cargo.toml missing icarus dependency".to_string());
        }

        Ok(())
    }

    /// Validate lib.rs content
    pub fn validate_lib_rs(&self) -> Result<(), String> {
        let content = self.read_file("src/lib.rs")
            .map_err(|e| format!("Could not read src/lib.rs: {}", e))?;

        if !content.contains("use icarus") {
            return Err("lib.rs should use icarus".to_string());
        }

        if content.len() < 50 {
            return Err("lib.rs appears to have insufficient content".to_string());
        }

        Ok(())
    }

    /// Run all validations
    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Err(e) = self.validate_basic_structure() {
            errors.push(e);
        }

        if let Err(e) = self.validate_cargo_toml() {
            errors.push(e);
        }

        if let Err(e) = self.validate_lib_rs() {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// MCP test helper for server management operations
pub struct McpTester {
    env: TestEnvironment,
}

impl McpTester {
    /// Create a new MCP tester
    pub fn new() -> Self {
        Self {
            env: TestEnvironment::new(),
        }
    }

    /// Get reference to the test environment
    pub fn env(&self) -> &TestEnvironment {
        &self.env
    }

    /// Create a mock MCP configuration
    pub fn create_mock_config(&self, servers: Vec<MockServer>) {
        let config = serde_json::json!({
            "servers": servers.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "version": "1.0.0"
        });
        
        fs::write(
            self.env.mcp_config_path(),
            serde_json::to_string_pretty(&config).unwrap()
        ).expect("Failed to write mock config");
    }

    /// List MCP servers
    pub fn list_servers(&self) -> assert_cmd::assert::Assert {
        self.env.icarus_cmd()
            .args(["--quiet", "mcp", "list"])
            .assert()
    }

    /// Add an MCP server
    pub fn add_server(&self, canister_id: &str, client: &str) -> assert_cmd::assert::Assert {
        self.env.icarus_cmd()
            .args([
                "mcp",
                "add",
                canister_id,
                "--client",
                client,
                "--skip-verify",
                "--yes",
            ])
            .assert()
    }

    /// Remove an MCP server
    pub fn remove_server(&self, name: &str) -> assert_cmd::assert::Assert {
        self.env.icarus_cmd()
            .args(["mcp", "remove", name, "--yes"])
            .assert()
    }

    /// Get MCP server status
    pub fn server_status(&self, name: &str) -> assert_cmd::assert::Assert {
        self.env.icarus_cmd()
            .args(["--quiet", "mcp", "status", name])
            .assert()
    }
}

/// Mock MCP server for testing
#[derive(Clone)]
pub struct MockServer {
    pub name: String,
    pub canister_id: String,
    pub network: String,
    pub client: String,
    pub enabled: bool,
    pub url: String,
    pub port: Option<u16>,
}

impl MockServer {
    /// Create a new mock server
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            canister_id: "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
            network: "local".to_string(),
            client: "claude-desktop".to_string(),
            enabled: true,
            url: format!("http://localhost:3000/mcp/{}", name),
            port: Some(3000),
        }
    }

    /// Set the client
    pub fn with_client(mut self, client: &str) -> Self {
        self.client = client.to_string();
        self
    }

    /// Set the network
    pub fn with_network(mut self, network: &str) -> Self {
        self.network = network.to_string();
        self
    }

    /// Disable the server
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Convert to JSON for configuration
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "canister_id": self.canister_id,
            "network": self.network,
            "url": self.url,
            "client": self.client,
            "port": self.port,
            "enabled": self.enabled,
            "created_at": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }
}

/// Test utilities for build operations
pub struct BuildTester {
    project_path: PathBuf,
}

impl BuildTester {
    /// Create a new build tester
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Check if cargo is available
    pub fn cargo_available() -> bool {
        which::which("cargo").is_ok()
    }

    /// Check if dfx is available
    pub fn dfx_available() -> bool {
        which::which("dfx").is_ok()
    }

    /// Build the project with specified options
    pub fn build(&self, env: &TestEnvironment, args: &[&str]) -> assert_cmd::assert::Assert {
        let mut cmd = env.icarus_cmd();
        cmd.current_dir(&self.project_path);
        cmd.arg("build");
        cmd.args(args);
        cmd.timeout(std::time::Duration::from_secs(120));
        cmd.assert()
    }

    /// Check if build artifacts exist
    pub fn has_artifacts(&self) -> bool {
        let target_dir = self.project_path.join("target");
        if !target_dir.exists() {
            return false;
        }

        // Look for WASM files in target directory
        self.find_wasm_files().is_some()
    }

    /// Find WASM files in the target directory
    pub fn find_wasm_files(&self) -> Option<Vec<PathBuf>> {
        let target_dir = self.project_path.join("target");
        let wasm_dir = target_dir.join("wasm32-unknown-unknown");
        
        if !wasm_dir.exists() {
            return None;
        }

        let mut wasm_files = Vec::new();
        
        // Search in both debug and release directories
        for mode in &["debug", "release"] {
            let mode_dir = wasm_dir.join(mode);
            if let Ok(entries) = fs::read_dir(mode_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                        wasm_files.push(path);
                    }
                }
            }
        }

        if wasm_files.is_empty() {
            None
        } else {
            Some(wasm_files)
        }
    }
}

/// Performance measurement utilities
pub struct PerformanceTester {
    start_time: std::time::Instant,
}

impl PerformanceTester {
    /// Start performance measurement
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Assert that operation completed within time limit
    pub fn assert_within(&self, limit: std::time::Duration, operation: &str) {
        let elapsed = self.elapsed();
        assert!(
            elapsed <= limit,
            "{} took too long: {:?} (limit: {:?})",
            operation,
            elapsed,
            limit
        );
    }

    /// Print performance result
    pub fn report(&self, operation: &str) {
        println!("✅ {} completed in {:?}", operation, self.elapsed());
    }
}

/// Cleanup utilities for tests
pub struct TestCleanup;

impl TestCleanup {
    /// Clean up temporary files and directories
    pub fn cleanup_temp_files(path: &Path) {
        if path.exists() {
            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!("Warning: Failed to cleanup {}: {}", path.display(), e);
            }
        }
    }

    /// Clean up environment variables
    pub fn cleanup_env_vars(vars: &[&str]) {
        for var in vars {
            std::env::remove_var(var);
        }
    }

    /// Reset cargo cache if needed
    pub fn reset_cargo_cache() {
        if let Ok(home) = std::env::var("HOME") {
            let cargo_dir = Path::new(&home).join(".cargo");
            if cargo_dir.exists() {
                // Don't actually remove cargo cache in tests, just note it exists
                println!("ℹ️  Cargo cache exists at {}", cargo_dir.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_setup() {
        let env = TestEnvironment::new();
        assert!(env.temp_dir.path().exists());
        assert!(env.config_dir.exists());
    }

    #[test]
    fn test_mock_server_creation() {
        let server = MockServer::new("test-server")
            .with_client("claude-code")
            .with_network("ic")
            .disabled();

        assert_eq!(server.name, "test-server");
        assert_eq!(server.client, "claude-code");
        assert_eq!(server.network, "ic");
        assert!(!server.enabled);
    }

    #[test]
    fn test_performance_measurement() {
        let perf = PerformanceTester::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let elapsed = perf.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(10));
        assert!(elapsed < std::time::Duration::from_millis(100));
    }
}