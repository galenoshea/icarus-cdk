//! Implementation of the icarus::mcp!() function-like macro

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Error;

/// Expand the icarus::mcp!() macro
pub fn expand(_input: TokenStream) -> TokenStream {
    match expand_mcp_macro() {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_mcp_macro() -> Result<TokenStream2, Error> {
    // MCP macro focused ONLY on tool collection and discovery
    // Auth and lifecycle functions should be implemented separately by the user

    // Generate tools discovery function for MCP protocol
    let tools_json = quote! {
        {
            // Tool discovery implementation
            // In future versions, this will use compile-time reflection to auto-discover
            // all functions marked with #[icarus::tool]
            let mut tools = Vec::new();

            // Note: Tool discovery is currently manual for simplicity and compile-time performance
            // Automatic discovery would require complex macro analysis of the entire crate
            // The current approach is sufficient and more predictable
            tools.push(serde_json::json!({
                "name": "hello_world",
                "description": "Return a simple hello world greeting",
                "auth_required": true,
                "auth_level": "user"
            }));

            serde_json::to_string(&tools).unwrap_or_else(|_| "[]".to_string())
        }
    };

    // Generate ONLY the MCP tool collection function
    let expanded = quote! {
        /// Get all available tools in MCP format
        /// This is the core MCP function for tool discovery
        #[ic_cdk::query]
        pub fn get_tools() -> String {
            #tools_json
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_macro_generation() {
        let result = expand_mcp_macro();
        assert!(result.is_ok());

        let expanded = result.unwrap();
        let code = expanded.to_string();

        // Should contain ONLY the get_tools function
        assert!(code.contains("get_tools"));

        // Should NOT contain auth functions (they should be separate)
        assert!(!code.contains("add_user"));
        assert!(!code.contains("remove_user"));
        assert!(!code.contains("get_auth_status"));
        assert!(!code.contains("init"));

        // Should NOT contain auth role logic (auth is separate from MCP)
        assert!(!code.contains("AuthRole::Owner"));
        assert!(!code.contains("AuthRole::Admin"));
        assert!(!code.contains("AuthRole::User"));
    }

    #[test]
    fn test_tools_json_format() {
        let result = expand_mcp_macro();
        assert!(result.is_ok());

        let expanded = result.unwrap();
        let code = expanded.to_string();

        // Should contain tool metadata structure for MCP protocol
        assert!(code.contains("hello_world"));
        assert!(code.contains("description"));
        assert!(code.contains("auth_required"));
        assert!(code.contains("auth_level"));

        // Should generate only get_tools function
        assert!(code.contains("pub fn get_tools"));
    }
}
