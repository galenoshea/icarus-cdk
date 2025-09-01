//! Tool registry and dispatch system

use icarus_core::error::ToolError;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Function pointer type for tool implementations
pub type ToolFunction = Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + 'static>> + 'static>;

/// Tool metadata for registration
pub struct ToolRegistration {
    pub name: String,
    pub description: String,
    pub function: ToolFunction,
}

/// Global tool registry
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolRegistration>,
}

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