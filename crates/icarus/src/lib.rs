//! Icarus SDK - Build MCP servers that run on the Internet Computer
//! 
//! Icarus SDK enables developers to create Model Context Protocol (MCP) servers
//! that run as Internet Computer Protocol (ICP) canisters, combining AI tool
//! integration with blockchain persistence.

// Re-export all subcrates
pub use icarus_core as core;
pub use icarus_derive as derive;
pub use icarus_canister as canister;
pub use icarus_types as types;

// Re-export commonly used items
pub use icarus_core::{
    IcarusError, IcarusServer, IcarusTool, IcarusResource,
    error::ToolError,
    response::{ToolSuccess, ToolStatus, tool_success, tool_ok}
};
pub use icarus_derive::{IcarusTool, icarus_server, icarus_tools, icarus_tool, IcarusStorable};

// Re-export storage utilities
pub mod storage {
    pub use icarus_canister::storage::*;
}

// Re-export test utilities
#[cfg(feature = "test-utils")]
pub mod test {
    pub use icarus_test::{
        TestContext, 
        test_utils::{assert_success, assert_error},
    };
    
    // Helper to create tool call requests
    pub fn tool_call_request(tool_name: &str, args: serde_json::Value) -> icarus_core::protocol::IcarusMcpRequest {
        icarus_core::protocol::IcarusMcpRequest {
            id: Some("1".to_string()),
            method: "tools/call".to_string(),
            params: serde_json::to_string(&serde_json::json!({
                "name": tool_name,
                "arguments": args
            })).unwrap(),
        }
    }
}

// Re-export key dependencies
pub use rmcp;
pub use ic_cdk;
pub use candid;

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        IcarusError,
        ToolError,
        IcarusServer,
        IcarusTool,
        IcarusResource,
        IcarusTool as DeriveTool,
        icarus_server,
        icarus_tools,
        icarus_tool,
        IcarusStorable,
        ToolSuccess,
        ToolStatus,
        tool_success,
        tool_ok,
    };
    
    // Common types needed for development
    pub use serde::{Serialize, Deserialize as SerdeDeserialize};
    pub use candid::{CandidType, Deserialize};
    pub use ic_cdk::api;
    
    // Re-export json! macro for convenience
    pub use serde_json::json;
    
    // Type aliases for common return types
    /// Result type for tool execution that bridges ICP and MCP
    /// 
    /// Tools can return any serializable type, which will be automatically
    /// converted to JSON for the MCP protocol. The default type parameter
    /// is `serde_json::Value` for maximum flexibility.
    pub type ToolResult<T = serde_json::Value> = Result<T, crate::ToolError>;
}

/// Generate Candid interface for the canister
#[macro_export]
macro_rules! export_candid {
    () => {
        // Generate the ic_cdk export_candid first
        ic_cdk::export_candid!();
        
        #[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
        fn export_candid() -> String {
            __export_service()
        }
    };
}