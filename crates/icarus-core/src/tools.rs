//! Tool registry and dispatch system

#[cfg(feature = "canister")]
use crate::error::ToolError;
#[cfg(feature = "canister")]
use candid::{CandidType, Deserialize};
#[cfg(feature = "canister")]
use serde::Serialize;
#[cfg(feature = "canister")]
use serde_json::Value;
#[cfg(feature = "canister")]
use std::collections::HashMap;
#[cfg(feature = "canister")]
use std::future::Future;
#[cfg(feature = "canister")]
use std::pin::Pin;

/// Function pointer type for tool implementations
#[cfg(feature = "canister")]
pub type ToolFunction = Box<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + 'static>> + 'static,
>;

/// Tool metadata for registration
#[cfg(feature = "canister")]
pub struct ToolRegistration {
    pub name: String,
    pub description: String,
    pub function: ToolFunction,
}

/// Tool information for MCP protocol (required by derive macros)
#[cfg(feature = "canister")]
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: String, // Store as JSON string for Candid compatibility
    pub output_schema: String, // Store as JSON string for Candid compatibility
}

/// Create JSON schema for a type (simplified implementation)
#[cfg(feature = "canister")]
pub fn create_schema_for<T>() -> String
where
    T: ?Sized,
{
    // This is a simplified schema generator
    // In a real implementation, you might use a library like schemars
    serde_json::json!({
        "type": "object",
        "description": "Type schema (auto-generated)"
    })
    .to_string()
}

/// Global tool registry
#[cfg(feature = "canister")]
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolRegistration>,
}

#[cfg(feature = "canister")]
impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register(&mut self, registration: ToolRegistration) {
        self.tools.insert(registration.name.clone(), registration);
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        match self.tools.get(name) {
            Some(tool) => {
                let fut = (tool.function)(args);
                fut.await
            }
            None => Err(ToolError::not_found(format!("Tool '{}' not found", name))),
        }
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<(String, String)> {
        self.tools
            .iter()
            .map(|(name, reg)| (name.clone(), reg.description.clone()))
            .collect()
    }
}

/// Macro to generate tool registration code
#[cfg(feature = "canister")]
#[macro_export]
macro_rules! register_tools {
    ($registry:expr, $server:ty, $($tool_name:literal => $method:ident),* $(,)?) => {
        $(
            {
                let registration = $crate::tools::ToolRegistration {
                    name: $tool_name.to_string(),
                    description: format!("{} tool", $tool_name),
                    function: Box::new(move |args: serde_json::Value| {
                        Box::pin(async move {
                            // Parse arguments and call the method
                            let server = <$server>::new();
                            server.$method(args).await
                        })
                    }),
                };
                $registry.register(registration);
            }
        )*
    };
}
