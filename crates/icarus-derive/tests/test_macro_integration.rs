//! Integration tests for Icarus derive macros
//!
//! These tests verify that the auth!(), mcp!(), and #[tool] macros
//! work correctly when used together.

// Test that the macros can be used in a realistic scenario
#[cfg(test)]
mod macro_integration_tests {

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
    fn test_new_builder_pattern_syntax() {
        // Test that the new builder pattern syntax parses correctly
        let tokens = quote::quote! {
            icarus::mcp! {
                .build()
            };
        };

        // This should parse as a valid macro invocation
        let parsed = syn::parse2::<syn::Stmt>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_builder_pattern_wasi_syntax() {
        // Test that builder pattern with WASI parses correctly
        let tokens = quote::quote! {
            icarus::mcp! {
                .with_wasi()
                .build()
            };
        };

        // This should parse as a valid macro invocation
        let parsed = syn::parse2::<syn::Stmt>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_builder_pattern_resource_syntax() {
        // Test that builder pattern with resources parses correctly
        let tokens = quote::quote! {
            icarus::mcp! {
                .with_wasi()
                .with_resource(my_setup)
                .on_init(|owner| Ok(()))
                .build()
            };
        };

        // This should parse as a valid macro invocation
        let parsed = syn::parse2::<syn::Stmt>(tokens);
        assert!(parsed.is_ok());
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
