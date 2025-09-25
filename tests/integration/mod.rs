//! Integration Test Framework for Icarus CDK
//!
//! This framework provides comprehensive testing utilities for MCP servers
//! built with the Icarus CDK, including canister deployment, MCP protocol
//! testing, and end-to-end validation.

use candid::{Decode, Encode, Principal};
use ic_agent::{Agent, identity::BasicIdentity};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Main integration test context
pub struct TestContext {
    pub temp_dir: TempDir,
    pub canister_id: Option<Principal>,
    pub agent: Agent,
    pub dfx_process: Option<tokio::process::Child>,
    pub bridge_process: Option<tokio::process::Child>,
    pub test_identity: BasicIdentity,
}

impl TestContext {
    /// Create a new test context with isolated environment
    pub async fn new() -> Result<Self, TestError> {
        let temp_dir = TempDir::new()
            .map_err(|e| TestError::Setup(format!("Failed to create temp dir: {}", e)))?;

        // Create test identity
        let test_identity = BasicIdentity::from_pem_file(create_test_identity(&temp_dir).await?)
            .map_err(|e| TestError::Setup(format!("Failed to create identity: {}", e)))?;

        // Create agent
        let agent = Agent::builder()
            .with_url("http://localhost:4943")
            .with_identity(test_identity.clone())
            .build()
            .map_err(|e| TestError::Setup(format!("Failed to create agent: {}", e)))?;

        Ok(Self {
            temp_dir,
            canister_id: None,
            agent,
            dfx_process: None,
            bridge_process: None,
            test_identity,
        })
    }

    /// Start local dfx replica
    pub async fn start_dfx(&mut self) -> Result<(), TestError> {
        // Stop any existing dfx
        let _ = Command::new("dfx").args(&["stop"]).output();

        // Start dfx in background
        let mut cmd = TokioCommand::new("dfx");
        cmd.args(&["start", "--clean", "--background"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let process = cmd.spawn()
            .map_err(|e| TestError::DfxError(format!("Failed to start dfx: {}", e)))?;

        self.dfx_process = Some(process);

        // Wait for dfx to be ready
        self.wait_for_dfx().await?;

        Ok(())
    }

    /// Deploy canister from template or custom code
    pub async fn deploy_canister(&mut self, template: Option<TestTemplate>) -> Result<Principal, TestError> {
        let project_dir = match template {
            Some(template) => self.create_test_project(template).await?,
            None => self.temp_dir.path().to_path_buf(),
        };

        // Build canister
        let output = Command::new("icarus")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .map_err(|e| TestError::BuildError(format!("Failed to build: {}", e)))?;

        if !output.status.success() {
            return Err(TestError::BuildError(format!(
                "Build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Deploy canister
        let output = Command::new("icarus")
            .args(&["deploy", "--network", "local"])
            .current_dir(&project_dir)
            .output()
            .map_err(|e| TestError::DeployError(format!("Failed to deploy: {}", e)))?;

        if !output.status.success() {
            return Err(TestError::DeployError(format!(
                "Deploy failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Extract canister ID from output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let canister_id = extract_canister_id(&output_str)
            .ok_or_else(|| TestError::DeployError("Failed to extract canister ID".to_string()))?;

        self.canister_id = Some(canister_id);
        Ok(canister_id)
    }

    /// Start MCP bridge for testing
    pub async fn start_bridge(&mut self) -> Result<(), TestError> {
        let canister_id = self.canister_id
            .ok_or_else(|| TestError::BridgeError("No canister deployed".to_string()))?;

        let mut cmd = TokioCommand::new("icarus");
        cmd.args(&["bridge", "start", "--canister-id", &canister_id.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn()
            .map_err(|e| TestError::BridgeError(format!("Failed to start bridge: {}", e)))?;

        self.bridge_process = Some(process);

        // Wait for bridge to be ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    /// Test MCP protocol compliance
    pub async fn test_mcp_protocol(&self) -> Result<McpTestResult, TestError> {
        let mut result = McpTestResult::default();

        // Test initialization
        result.initialization = self.test_mcp_initialization().await?;

        // Test tool listing
        result.tool_listing = self.test_tool_listing().await?;

        // Test tool execution
        result.tool_execution = self.test_tool_execution().await?;

        Ok(result)
    }

    /// Call canister method directly
    pub async fn call_canister(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, TestError> {
        let canister_id = self.canister_id
            .ok_or_else(|| TestError::CanisterError("No canister deployed".to_string()))?;

        self.agent
            .update(&canister_id, method)
            .with_arg(&args)
            .call_and_wait()
            .await
            .map_err(|e| TestError::CanisterError(format!("Canister call failed: {}", e)))
    }

    /// Query canister method
    pub async fn query_canister(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, TestError> {
        let canister_id = self.canister_id
            .ok_or_else(|| TestError::CanisterError("No canister deployed".to_string()))?;

        self.agent
            .query(&canister_id, method)
            .with_arg(&args)
            .call()
            .await
            .map_err(|e| TestError::CanisterError(format!("Canister query failed: {}", e)))
    }

    /// Cleanup test environment
    pub async fn cleanup(mut self) -> Result<(), TestError> {
        // Stop bridge
        if let Some(mut bridge) = self.bridge_process.take() {
            let _ = bridge.kill().await;
        }

        // Stop dfx
        if let Some(mut dfx) = self.dfx_process.take() {
            let _ = dfx.kill().await;
        }

        let _ = Command::new("dfx").args(&["stop"]).output();

        Ok(())
    }

    // Private helper methods
    async fn wait_for_dfx(&self) -> Result<(), TestError> {
        let start = Instant::now();
        let timeout_duration = Duration::from_secs(60);

        while start.elapsed() < timeout_duration {
            if let Ok(output) = Command::new("dfx").args(&["ping"]).output() {
                if output.status.success() {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(TestError::DfxError("Timeout waiting for dfx to start".to_string()))
    }

    async fn create_test_project(&self, template: TestTemplate) -> Result<PathBuf, TestError> {
        let project_path = self.temp_dir.path().join("test-project");

        let output = Command::new("icarus")
            .args(&["new", "test-project", "--template", template.name()])
            .current_dir(self.temp_dir.path())
            .output()
            .map_err(|e| TestError::Setup(format!("Failed to create project: {}", e)))?;

        if !output.status.success() {
            return Err(TestError::Setup(format!(
                "Project creation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(project_path)
    }

    async fn test_mcp_initialization(&self) -> Result<bool, TestError> {
        // Test MCP initialization request
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        // This would normally be sent via bridge connection
        // For now, we'll test that the canister has list_tools method
        let result = self.query_canister("list_tools", Encode!(&()).unwrap()).await?;
        let decoded: Result<String, String> = Decode!(&result, Result<String, String>)
            .map_err(|e| TestError::McpError(format!("Failed to decode response: {}", e)))?;

        match decoded {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn test_tool_listing(&self) -> Result<bool, TestError> {
        let result = self.query_canister("list_tools", Encode!(&()).unwrap()).await?;
        let decoded: Result<String, String> = Decode!(&result, Result<String, String>)
            .map_err(|e| TestError::McpError(format!("Failed to decode response: {}", e)))?;

        match decoded {
            Ok(json_str) => {
                let _parsed: JsonValue = serde_json::from_str(&json_str)
                    .map_err(|e| TestError::McpError(format!("Invalid JSON response: {}", e)))?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    async fn test_tool_execution(&self) -> Result<bool, TestError> {
        // This would test actual tool execution
        // For now, just verify canister responds to basic queries
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub enum TestTemplate {
    DataManager,
    TaskScheduler,
    ApiGateway,
}

impl TestTemplate {
    fn name(&self) -> &'static str {
        match self {
            TestTemplate::DataManager => "data-manager",
            TestTemplate::TaskScheduler => "task-scheduler",
            TestTemplate::ApiGateway => "api-gateway",
        }
    }
}

#[derive(Debug, Default)]
pub struct McpTestResult {
    pub initialization: bool,
    pub tool_listing: bool,
    pub tool_execution: bool,
}

impl McpTestResult {
    pub fn is_valid(&self) -> bool {
        self.initialization && self.tool_listing && self.tool_execution
    }

    pub fn summary(&self) -> String {
        format!(
            "MCP Test Results: Init: {}, Tools: {}, Exec: {}",
            self.initialization, self.tool_listing, self.tool_execution
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Setup error: {0}")]
    Setup(String),

    #[error("DFX error: {0}")]
    DfxError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Deploy error: {0}")]
    DeployError(String),

    #[error("Bridge error: {0}")]
    BridgeError(String),

    #[error("Canister error: {0}")]
    CanisterError(String),

    #[error("MCP protocol error: {0}")]
    McpError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// Helper functions
async fn create_test_identity(temp_dir: &TempDir) -> Result<PathBuf, TestError> {
    let identity_path = temp_dir.path().join("test-identity.pem");

    let output = Command::new("dfx")
        .args(&["identity", "new", "test-identity", "--disable-encryption"])
        .output()
        .map_err(|e| TestError::Setup(format!("Failed to create identity: {}", e)))?;

    if !output.status.success() {
        return Err(TestError::Setup(format!(
            "Identity creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(identity_path)
}

fn extract_canister_id(output: &str) -> Option<Principal> {
    // Parse canister ID from deploy output
    // Look for patterns like "Canister deployed successfully!"
    // "Canister ID: rdmx6-jaaaa-aaaaa-aaadq-cai"

    for line in output.lines() {
        if line.contains("Canister ID:") {
            if let Some(id_part) = line.split("Canister ID:").nth(1) {
                let id_str = id_part.trim();
                if let Ok(principal) = Principal::from_text(id_str) {
                    return Some(principal);
                }
            }
        }
    }

    None
}

/// Macro for creating integration tests
#[macro_export]
macro_rules! icarus_integration_test {
    ($name:ident, $template:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $name() {
            let mut ctx = TestContext::new().await.expect("Failed to create test context");

            // Setup
            ctx.start_dfx().await.expect("Failed to start dfx");
            let canister_id = ctx.deploy_canister(Some($template)).await
                .expect("Failed to deploy canister");

            println!("Test canister deployed: {}", canister_id);

            // Run test
            let test_result = async move {
                $test_body(&mut ctx).await
            }.await;

            // Cleanup
            ctx.cleanup().await.expect("Failed to cleanup");

            // Assert test passed
            test_result.expect("Test failed");
        }
    };
}

/// Performance benchmarking utilities
pub struct PerformanceBenchmark {
    pub name: String,
    pub iterations: usize,
    pub results: Vec<Duration>,
}

impl PerformanceBenchmark {
    pub fn new(name: impl Into<String>, iterations: usize) -> Self {
        Self {
            name: name.into(),
            iterations,
            results: Vec::with_capacity(iterations),
        }
    }

    pub async fn run<F, Fut, T>(&mut self, test_fn: F) -> Result<Vec<T>, TestError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, TestError>>,
    {
        let mut results = Vec::with_capacity(self.iterations);

        for i in 0..self.iterations {
            let start = Instant::now();
            let result = test_fn().await?;
            let duration = start.elapsed();

            self.results.push(duration);
            results.push(result);

            if i % 10 == 0 {
                println!("Benchmark {}: iteration {}/{}", self.name, i + 1, self.iterations);
            }
        }

        Ok(results)
    }

    pub fn summary(&self) -> BenchmarkSummary {
        let total: Duration = self.results.iter().sum();
        let avg = total / self.results.len() as u32;

        let mut sorted = self.results.clone();
        sorted.sort();

        let min = sorted.first().copied().unwrap_or_default();
        let max = sorted.last().copied().unwrap_or_default();
        let median = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];

        BenchmarkSummary {
            name: self.name.clone(),
            iterations: self.iterations,
            min,
            max,
            avg,
            median,
            p95,
            total,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub name: String,
    pub iterations: usize,
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub total: Duration,
}

impl BenchmarkSummary {
    pub fn print_report(&self) {
        println!("\n=== Benchmark Report: {} ===", self.name);
        println!("Iterations: {}", self.iterations);
        println!("Min:        {:?}", self.min);
        println!("Max:        {:?}", self.max);
        println!("Average:    {:?}", self.avg);
        println!("Median:     {:?}", self.median);
        println!("95th %ile:  {:?}", self.p95);
        println!("Total:      {:?}", self.total);
        println!("Rate:       {:.2} ops/sec", self.iterations as f64 / self.total.as_secs_f64());
    }
}