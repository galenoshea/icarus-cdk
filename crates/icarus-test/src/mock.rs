//! Mock implementations for testing

use ic_cdk::api::call::RejectionCode as RejectCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock environment for testing canister functionality
pub struct MockEnvironment {
    /// Current time in nanoseconds
    pub time: Arc<Mutex<u64>>,
    /// Stable memory contents
    pub stable_memory: Arc<Mutex<Vec<u8>>>,
    /// Call results
    pub call_results: Arc<Mutex<HashMap<String, Result<Vec<u8>, (RejectCode, String)>>>>,
}

impl MockEnvironment {
    /// Create a new mock environment
    pub fn new() -> Self {
        Self {
            time: Arc::new(Mutex::new(0)),
            stable_memory: Arc::new(Mutex::new(Vec::new())),
            call_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Set the current time
    pub fn set_time(&self, time: u64) {
        *self.time.lock().unwrap() = time;
    }
    
    /// Advance time by the given number of nanoseconds
    pub fn advance_time(&self, nanos: u64) {
        *self.time.lock().unwrap() += nanos;
    }
    
    /// Get the current time
    pub fn get_time(&self) -> u64 {
        *self.time.lock().unwrap()
    }
    
    /// Set expected call result
    pub fn expect_call(
        &self,
        method: String,
        result: Result<Vec<u8>, (RejectCode, String)>,
    ) {
        self.call_results.lock().unwrap().insert(method, result);
    }
}

/// Mock implementation of ICP system API for testing
pub mod sys_api {
    use super::*;
    
    thread_local! {
        static MOCK_ENV: Arc<Mutex<Option<MockEnvironment>>> = Arc::new(Mutex::new(None));
    }
    
    /// Set the mock environment for this thread
    pub fn set_mock_env(env: MockEnvironment) {
        MOCK_ENV.with(|e| {
            *e.lock().unwrap() = Some(env);
        });
    }
    
    /// Get current time from mock environment
    pub fn time() -> u64 {
        MOCK_ENV.with(|e| {
            e.lock()
                .unwrap()
                .as_ref()
                .map(|env| env.get_time())
                .unwrap_or(0)
        })
    }
}

/// Builder for creating mock canister states
pub struct MockStateBuilder {
    tools: Vec<Box<dyn icarus_core::tool::IcarusTool>>,
    resources: Vec<Box<dyn icarus_core::resource::IcarusResource>>,
}

impl MockStateBuilder {
    /// Create a new mock state builder
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            resources: Vec::new(),
        }
    }
    
    /// Add a tool to the mock state
    pub fn with_tool(mut self, tool: Box<dyn icarus_core::tool::IcarusTool>) -> Self {
        self.tools.push(tool);
        self
    }
    
    /// Add a resource to the mock state
    pub fn with_resource(mut self, resource: Box<dyn icarus_core::resource::IcarusResource>) -> Self {
        self.resources.push(resource);
        self
    }
    
    /// Build the mock state
    pub fn build(self) -> MockCanisterState {
        MockCanisterState {
            tools: self.tools,
            resources: self.resources,
        }
    }
}

/// Mock canister state for testing
pub struct MockCanisterState {
    tools: Vec<Box<dyn icarus_core::tool::IcarusTool>>,
    resources: Vec<Box<dyn icarus_core::resource::IcarusResource>>,
}

impl MockCanisterState {
    /// Get all tools
    pub fn tools(&self) -> &[Box<dyn icarus_core::tool::IcarusTool>] {
        &self.tools
    }
    
    /// Get all resources
    pub fn resources(&self) -> &[Box<dyn icarus_core::resource::IcarusResource>] {
        &self.resources
    }
}