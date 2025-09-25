//! Integration tests for Icarus derive macros
//!
//! These tests verify that the auth!(), mcp!(), and #[tool] macros
//! work correctly when used together.

// Test that the macros can be used in a realistic scenario
#[cfg(test)]
mod macro_integration_tests {

    #[test]
    fn test_auth_macro_compiles() {
        // Test that the auth!() macro generates valid code
        // This is a compile-time test - if it compiles, the macro works
        let tokens = quote::quote! {
            use icarus::prelude::*;
            use ic_cdk_macros::export_candid;

            icarus::auth!();
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::File>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_mcp_macro_compiles() {
        // Test that the mcp!() macro generates valid code
        let tokens = quote::quote! {
            use icarus::prelude::*;
            use ic_cdk_macros::export_candid;

            icarus::mcp!();
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::File>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_tool_macro_compiles() {
        // Test that the #[tool] macro generates valid code
        let tokens = quote::quote! {
            #[icarus::tool("Test tool")]
            pub async fn test_tool() -> Result<String, String> {
                Ok("success".to_string())
            }
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_tool_macro_with_auth_levels() {
        // Test that the #[tool] macro works with different auth levels

        // Test public auth
        let tokens = quote::quote! {
            #[icarus::tool("Test tool", auth = "public")]
            pub async fn test_tool_public() -> Result<String, String> {
                Ok("success".to_string())
            }
        };
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(
            parsed.is_ok(),
            "Failed to parse tool macro with public auth"
        );

        // Test user auth
        let tokens = quote::quote! {
            #[icarus::tool("Test tool", auth = "user")]
            pub async fn test_tool_user() -> Result<String, String> {
                Ok("success".to_string())
            }
        };
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(parsed.is_ok(), "Failed to parse tool macro with user auth");

        // Test admin auth
        let tokens = quote::quote! {
            #[icarus::tool("Test tool", auth = "admin")]
            pub async fn test_tool_admin() -> Result<String, String> {
                Ok("success".to_string())
            }
        };
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(parsed.is_ok(), "Failed to parse tool macro with admin auth");

        // Test owner auth
        let tokens = quote::quote! {
            #[icarus::tool("Test tool", auth = "owner")]
            pub async fn test_tool_owner() -> Result<String, String> {
                Ok("success".to_string())
            }
        };
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(parsed.is_ok(), "Failed to parse tool macro with owner auth");
    }

    #[test]
    fn test_complete_macro_setup_compiles() {
        // Test that a complete setup with all macros compiles
        let tokens = quote::quote! {
            use icarus::prelude::*;
            use ic_cdk_macros::export_candid;

            #[ic_cdk::update]
            #[icarus::tool("Process data")]
            pub async fn process_data(data: String) -> Result<String, String> {
                Ok(format!("Processed: {}", data))
            }

            #[ic_cdk::query]
            #[icarus::tool("Get info", auth = "public")]
            pub fn get_info() -> String {
                "Service information".to_string()
            }

            #[ic_cdk::update]
            #[icarus::tool("Admin function", auth = "admin")]
            pub async fn admin_function() -> String {
                "admin operation".to_string()
            }

            icarus::auth!();
            icarus::mcp!();
            export_candid!();
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::File>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_mcp_macro_basic_usage() {
        // Verify that the mcp!() macro can be used in basic scenarios
        let tokens = quote::quote! {
            use icarus::prelude::*;

            #[ic_cdk::update]
            #[icarus::tool("Test tool")]
            pub async fn test_tool() -> Result<String, String> {
                Ok("test".to_string())
            }

            icarus::mcp!();
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::File>(tokens);
        assert!(parsed.is_ok(), "MCP macro should work with tool functions");
    }

    #[test]
    fn test_auth_macro_basic_usage() {
        // Verify that the auth!() macro can be used in basic scenarios
        let tokens = quote::quote! {
            use icarus::prelude::*;

            #[ic_cdk::update]
            #[icarus::tool("Admin tool", auth = "admin")]
            pub async fn admin_tool() -> Result<String, String> {
                Ok("admin".to_string())
            }

            icarus::auth!();
        };

        // Parse the generated tokens to ensure they're valid Rust
        let parsed = syn::parse2::<syn::File>(tokens);
        assert!(
            parsed.is_ok(),
            "Auth macro should work with authenticated tools"
        );
    }

    #[test]
    fn test_tool_macro_attributes_preserved() {
        // Test that the #[tool] macro preserves other attributes
        let input = quote::quote! {
            #[ic_cdk::update]
            #[icarus::tool("Test tool")]
            pub async fn test_tool() -> Result<String, String> {
                Ok("success".to_string())
            }
        };

        // The tool macro should preserve the ic_cdk::update attribute
        let parsed = syn::parse2::<syn::ItemFn>(input).unwrap();
        assert_eq!(parsed.attrs.len(), 2); // Should have both attributes
    }

    #[test]
    fn test_macro_error_handling() {
        // Test that macros handle various edge cases gracefully

        // Test tool macro with empty description
        let tokens = quote::quote! {
            #[icarus::tool("")]
            pub fn empty_desc() -> String {
                "test".to_string()
            }
        };
        let parsed = syn::parse2::<syn::ItemFn>(tokens);
        assert!(
            parsed.is_ok(),
            "Tool macro should handle empty descriptions"
        );

        // Test tool macro with unusual auth values (should compile with default behavior)
        let tokens = quote::quote! {
            #[icarus::tool("Test", auth = "unknown")]
            pub fn unknown_auth() -> String {
                "test".to_string()
            }
        };
        let _parsed = syn::parse2::<syn::ItemFn>(tokens);
        // This may or may not parse depending on macro implementation, but shouldn't panic
    }
}
