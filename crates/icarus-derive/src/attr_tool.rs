//! Implementation of the #[icarus::tool] attribute macro
//!
//! This provides an attribute for marking functions as MCP tools with optional authentication.
//! Supports auth levels: none (default), user, admin

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemFn, LitStr, Token};

/// Auth level for tool functions
#[derive(Debug, Clone, PartialEq)]
enum AuthLevel {
    None,  // Public access (default)
    User,  // Requires authenticated user
    Admin, // Requires admin role
}

impl AuthLevel {
    /// Parse auth level from string literal
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "none" => Ok(AuthLevel::None),
            "user" => Ok(AuthLevel::User),
            "admin" => Ok(AuthLevel::Admin),
            _ => Err(format!(
                "Invalid auth level '{}'. Must be 'none', 'user', or 'admin'",
                s
            )),
        }
    }

    /// Generate auth check code for this level
    fn generate_auth_check(&self) -> proc_macro2::TokenStream {
        match self {
            AuthLevel::None => quote! {}, // No auth check needed
            AuthLevel::User => quote! {
                use icarus::prelude::*;
                require_role_or_higher(AuthRole::User);
            },
            AuthLevel::Admin => quote! {
                use icarus::prelude::*;
                require_role_or_higher(AuthRole::Admin);
            },
        }
    }
}

/// Tool arguments structure for parsing
struct ToolArgs {
    description: String,
    auth_level: AuthLevel,
}

impl Parse for ToolArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut auth_level = AuthLevel::None;

        // Parse first argument (description)
        let desc: LitStr = input.parse()?;
        let description = Some(desc.value());

        // Parse optional auth parameter
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            // Look for auth = "value" pattern
            if input.peek(syn::Ident) {
                let ident: syn::Ident = input.parse()?;
                if ident == "auth" {
                    input.parse::<Token![=]>()?;
                    let auth_str: LitStr = input.parse()?;
                    auth_level = AuthLevel::from_str(&auth_str.value())
                        .map_err(|e| syn::Error::new_spanned(&auth_str, e))?;
                } else {
                    return Err(syn::Error::new_spanned(ident, "Expected 'auth' parameter"));
                }
            }
        }

        let description = description
            .ok_or_else(|| syn::Error::new(input.span(), "Tool requires a description string"))?;

        Ok(ToolArgs {
            description,
            auth_level,
        })
    }
}

/// Parse tool attributes to extract description and auth level
fn parse_tool_attributes(args: TokenStream) -> Result<(String, AuthLevel), String> {
    if args.is_empty() {
        return Err("Tool requires a description string".to_string());
    }

    let tool_args =
        syn::parse::<ToolArgs>(args).map_err(|e| format!("Failed to parse arguments: {}", e))?;

    Ok((tool_args.description, tool_args.auth_level))
}

/// Expand the #[icarus::tool] attribute macro
///
/// Parses the tool description and optional auth level, then injects
/// appropriate authentication checks at the beginning of the function.
///
/// # Examples
///
/// ```rust,ignore
/// #[icarus::tool("Public function")]
/// #[icarus::tool("User function", auth = "user")]
/// #[icarus::tool("Admin function", auth = "admin")]
/// ```
pub fn expand(args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    // Parse tool attributes
    let (_description, auth_level) = match parse_tool_attributes(args) {
        Ok(attrs) => attrs,
        Err(err) => {
            return syn::Error::new_spanned(&function, format!("#[icarus::tool] error: {}", err))
                .to_compile_error()
                .into();
        }
    };

    // Generate auth check based on level
    let auth_check = auth_level.generate_auth_check();

    // Get function components
    let fn_vis = &function.vis;
    let fn_name = &function.sig.ident;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_attrs = &function.attrs;
    let fn_block = &function.block;
    let fn_asyncness = &function.sig.asyncness;
    let fn_generics = &function.sig.generics;

    // Inject auth check at the beginning of the function
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            #auth_check
            #fn_block
        }
    };

    expanded.into()
}
