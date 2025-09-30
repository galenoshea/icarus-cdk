//! Implementation of the #[tool] attribute macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, spanned::Spanned, ItemFn};

use crate::error::{MacroError, MacroResult};
use crate::utils::{
    extract_parameters, extract_return_type, generate_function_call,
    generate_json_schema_from_parameters, generate_param_struct_name, is_async_function,
};

/// Maximum number of parameters a tool function can have
const MAX_PARAMETERS: usize = 50;

/// Implementation of the #[tool] attribute macro.
pub(crate) fn tool_impl(args: TokenStream, input: TokenStream) -> MacroResult<TokenStream> {
    // Parse the function
    let function: ItemFn = parse2(input)?;

    // Parse tool configuration from macro arguments
    let tool_config = if args.is_empty() {
        ToolConfig::default()
    } else {
        parse_tool_args(args)
    };

    // Validate the function signature
    validate_function_signature(&function)?;

    // Extract function information
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_attrs = &function.attrs;
    let fn_sig = &function.sig;
    let fn_block = &function.block;
    let is_async = is_async_function(fn_sig);

    // Extract parameters and return type
    let parameters = extract_parameters(&function.sig.inputs)?;

    // Validate parameter count to prevent pathological cases
    if parameters.len() > MAX_PARAMETERS {
        return Err(MacroError::invalid_signature_spanned(
            format!(
                "Tool functions cannot have more than {} parameters (found {}). \
                 Large parameter counts lead to slow compilation and large generated code.",
                MAX_PARAMETERS,
                parameters.len()
            ),
            function.sig.span(),
        ));
    }

    let _return_type = extract_return_type(&function.sig.output);

    // Generate parameter structure
    let param_struct_name = generate_param_struct_name(fn_name);
    let param_struct = generate_parameter_struct(&param_struct_name, &parameters);

    // Generate tool wrapper function
    let wrapper_fn_name = format_ident!("{}_tool_wrapper", fn_name);
    let tool_wrapper = generate_tool_wrapper(
        &wrapper_fn_name,
        fn_name,
        &param_struct_name,
        &parameters,
        is_async,
        tool_config.auth_level.as_deref(),
    );

    // Generate tool registration
    let registration_fn_name = format_ident!("{}_tool_info", fn_name);
    let description = tool_config
        .description
        .or_else(|| extract_doc_comment(fn_attrs));

    // Determine the tool name (custom or default)
    let default_tool_name = fn_name.to_string();
    let tool_name = tool_config.name.as_deref().unwrap_or(&default_tool_name);

    let tool_registration = generate_tool_info_function(
        &registration_fn_name,
        tool_name,
        &parameters,
        description.as_deref(),
        tool_config.auth_level.as_deref(),
    );

    // Generate linkme registration for automatic tool discovery
    let tool_registry_item = generate_tool_registry_item(&registration_fn_name);

    // Generate executor registration for runtime tool execution
    let executor_registration =
        generate_executor_registration(tool_name, &wrapper_fn_name, is_async);

    // Keep the original function unchanged
    let original_function = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block
    };

    // Combine all generated code
    Ok(quote! {
        #original_function

        #param_struct

        #tool_wrapper

        #tool_registration

        #tool_registry_item

        #executor_registration
    })
}

/// Configuration options for the #[tool] attribute.
#[derive(Debug, Default)]
struct ToolConfig {
    /// Optional custom tool name (allows kebab-case names for MCP compatibility)
    name: Option<String>,
    /// Optional custom description
    description: Option<String>,
    /// Authentication level: "none", "user", or "admin"
    auth_level: Option<String>,
}

/// Parses tool attribute arguments.
fn parse_tool_args(args: TokenStream) -> ToolConfig {
    use syn::parse::{Parse, ParseStream};
    use syn::Token;

    struct ToolArgs {
        name: Option<String>,
        description: Option<String>,
        auth_level: Option<String>,
    }

    impl Parse for ToolArgs {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let mut name = None;
            let mut description = None;
            let mut auth_level = None;

            // Try to parse the first argument as a string literal (description)
            if input.peek(syn::LitStr) {
                let lit: syn::LitStr = input.parse()?;
                description = Some(lit.value());

                // Parse remaining comma-separated arguments
                while !input.is_empty() {
                    let _: Token![,] = input.parse()?;

                    if input.is_empty() {
                        break;
                    }

                    let ident: syn::Ident = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    let value: syn::LitStr = input.parse()?;

                    if ident == "auth" {
                        auth_level = Some(value.value());
                    } else if ident == "name" {
                        name = Some(value.value());
                    }
                }
            } else if input.peek(syn::Ident) {
                // Parse key=value pairs when no positional description
                while !input.is_empty() {
                    let ident: syn::Ident = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    let value: syn::LitStr = input.parse()?;

                    if ident == "name" {
                        name = Some(value.value());
                    } else if ident == "description" {
                        description = Some(value.value());
                    } else if ident == "auth" {
                        auth_level = Some(value.value());
                    }

                    // Check for trailing comma
                    if input.peek(Token![,]) {
                        let _: Token![,] = input.parse()?;
                    } else {
                        break;
                    }
                }
            }

            Ok(ToolArgs {
                name,
                description,
                auth_level,
            })
        }
    }

    let parsed = parse2::<ToolArgs>(args).unwrap_or(ToolArgs {
        name: None,
        description: None,
        auth_level: None,
    });

    ToolConfig {
        name: parsed.name,
        description: parsed.description,
        auth_level: parsed.auth_level,
    }
}

/// Validates that the function signature is suitable for a tool.
fn validate_function_signature(function: &ItemFn) -> MacroResult<()> {
    // Check for generic parameters
    if !function.sig.generics.params.is_empty() {
        return Err(MacroError::unsupported_feature_spanned(
            "Generic functions",
            "Tool functions cannot have generic parameters",
            function.sig.generics.span(),
        ));
    }

    // Check for lifetimes
    if function.sig.generics.lifetimes().count() > 0 {
        return Err(MacroError::unsupported_feature_spanned(
            "Lifetime parameters",
            "Tool functions cannot have lifetime parameters",
            function.sig.generics.span(),
        ));
    }

    // Check for self parameter
    for input in &function.sig.inputs {
        if let syn::FnArg::Receiver(receiver) = input {
            return Err(MacroError::invalid_signature_spanned(
                "Tool functions cannot have self parameters",
                receiver.span(),
            ));
        }
    }

    Ok(())
}

/// Generates a parameter structure for the tool.
fn generate_parameter_struct(
    struct_name: &syn::Ident,
    parameters: &[crate::utils::ParameterInfo],
) -> TokenStream {
    let field_definitions: Vec<TokenStream> = parameters
        .iter()
        .map(|param| {
            let name = &param.name;
            let ty = &param.ty;

            quote! {
                pub #name: #ty,
            }
        })
        .collect();

    quote! {
        #[derive(serde::Deserialize)]
        struct #struct_name {
            #(#field_definitions)*
        }
    }
}

/// Generates the tool wrapper function that handles MCP protocol.
///
/// This function generates two wrappers:
/// 1. Internal wrapper that returns `Result<String, String>`
/// 2. Executor wrapper that converts to `RuntimeResult<LegacyToolResult<'static>>`
#[allow(clippy::too_many_arguments)]
fn generate_tool_wrapper(
    wrapper_name: &syn::Ident,
    fn_name: &syn::Ident,
    param_struct_name: &syn::Ident,
    parameters: &[crate::utils::ParameterInfo],
    is_async: bool,
    auth_level: Option<&str>,
) -> TokenStream {
    let fn_call = generate_function_call(fn_name, parameters, is_async);

    // Generate auth check code if auth_level is specified
    let auth_check = match auth_level {
        Some("user") => quote! {
            {
                let caller = ::ic_cdk::caller();
                if !::icarus_core::auth::has_user_access(&caller) {
                    return Err("Authentication required: user or admin access needed".to_string());
                }
            }
        },
        Some("admin") => quote! {
            {
                let caller = ::ic_cdk::caller();
                if !::icarus_core::auth::has_admin_access(&caller) {
                    return Err("Authentication required: admin access needed".to_string());
                }
            }
        },
        _ => quote! {}, // "none" or no auth - no check needed
    };

    if is_async {
        quote! {
            async fn #wrapper_name(args_json: &str) -> Result<String, String> {
                #auth_check

                let args: #param_struct_name = serde_json::from_str(args_json)
                    .map_err(|e| format!("Invalid arguments: {e}"))?;

                let result = #fn_call;

                serde_json::to_string(&result)
                    .map_err(|e| format!("Failed to serialize result: {e}"))
            }
        }
    } else {
        quote! {
            fn #wrapper_name(args_json: &str) -> Result<String, String> {
                #auth_check

                let args: #param_struct_name = serde_json::from_str(args_json)
                    .map_err(|e| format!("Invalid arguments: {e}"))?;

                let result = #fn_call;

                serde_json::to_string(&result)
                    .map_err(|e| format!("Failed to serialize result: {e}"))
            }
        }
    }
}

/// Generates the tool information function for registration.
fn generate_tool_info_function(
    info_fn_name: &syn::Ident,
    tool_name: &str,
    parameters: &[crate::utils::ParameterInfo],
    description: Option<&str>,
    auth_level: Option<&str>,
) -> TokenStream {
    let default_description = format!("Tool: {tool_name}");
    let description = description.unwrap_or(&default_description);

    // Generate JSON Schema for input parameters
    let input_schema = generate_json_schema_from_parameters(parameters);

    // Generate annotations if auth_level is specified
    let annotations_code = if let Some(auth) = auth_level {
        // Map auth_level to RMCP ToolAnnotations hints
        let read_only = auth == "none"; // Public tools might be read-only

        quote! {
            let annotations = ::icarus_core::ToolAnnotations {
                title: None,
                read_only_hint: Some(#read_only),
                destructive_hint: None,
                idempotent_hint: None,
                open_world_hint: None,
            };
            tool = tool.annotate(annotations);
        }
    } else {
        quote! {}
    };

    quote! {
        fn #info_fn_name() -> ::icarus_core::Tool {
            let input_schema = #input_schema;

            let mut tool = ::icarus_core::Tool::new(
                #tool_name,
                #description,
                input_schema,
            );

            #annotations_code

            tool
        }
    }
}

/// Generates linkme registration for automatic tool discovery.
fn generate_tool_registry_item(info_fn_name: &syn::Ident) -> TokenStream {
    let registry_static_name =
        format_ident!("TOOL_{}_REGISTRY", info_fn_name.to_string().to_uppercase());

    quote! {
        #[::linkme::distributed_slice(::icarus_runtime::TOOL_REGISTRY)]
        static #registry_static_name: fn() -> ::icarus_core::Tool = #info_fn_name;
    }
}

/// Generates executor wrapper and registration for runtime tool execution.
///
/// This creates:
/// 1. An executor function that wraps the internal wrapper and converts results
/// 2. A registration function that registers the executor with the runtime
fn generate_executor_registration(
    tool_name: &str,
    wrapper_fn_name: &syn::Ident,
    is_async: bool,
) -> TokenStream {
    // Use the wrapper function name to derive executor names to avoid conflicts
    let executor_fn_name = format_ident!("{}_executor", wrapper_fn_name);
    // Generate UPPER_CASE static variable name for Rust conventions
    let registration_fn_name = format_ident!(
        "{}_REGISTRATION",
        wrapper_fn_name.to_string().to_uppercase()
    );

    if is_async {
        quote! {
            fn #executor_fn_name(args: &str) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::icarus_runtime::RuntimeResult<::icarus_core::LegacyToolResult<'static>>> + Send>> {
                let args = args.to_string();
                ::std::boxed::Box::pin(async move {
                    match #wrapper_fn_name(&args).await {
                        Ok(result_json) => {
                            Ok(::icarus_core::LegacyToolResult::success(::std::borrow::Cow::Owned(result_json)))
                        }
                        Err(error_msg) => {
                            Ok(::icarus_core::LegacyToolResult::error(::std::borrow::Cow::Owned(error_msg)))
                        }
                    }
                })
            }

            #[::linkme::distributed_slice(::icarus_runtime::EXECUTOR_INIT)]
            static #registration_fn_name: fn() = || {
                let tool_id = ::icarus_core::ToolId::new(#tool_name)
                    .unwrap_or_else(|e| {
                        ::std::panic!(
                            "Invalid tool name '{}': {}. Tool names must be valid identifiers.",
                            #tool_name, e
                        )
                    });

                let _ = ::icarus_runtime::ToolRegistry::register_async_executor(
                    tool_id,
                    #executor_fn_name
                );
            };
        }
    } else {
        quote! {
            fn #executor_fn_name(args: &str) -> ::icarus_runtime::RuntimeResult<::icarus_core::LegacyToolResult<'static>> {
                match #wrapper_fn_name(args) {
                    Ok(result_json) => {
                        Ok(::icarus_core::LegacyToolResult::success(::std::borrow::Cow::Owned(result_json)))
                    }
                    Err(error_msg) => {
                        Ok(::icarus_core::LegacyToolResult::error(::std::borrow::Cow::Owned(error_msg)))
                    }
                }
            }

            #[::linkme::distributed_slice(::icarus_runtime::EXECUTOR_INIT)]
            static #registration_fn_name: fn() = || {
                let tool_id = ::icarus_core::ToolId::new(#tool_name)
                    .unwrap_or_else(|e| {
                        ::std::panic!(
                            "Invalid tool name '{}': {}. Tool names must be valid identifiers.",
                            #tool_name, e
                        )
                    });

                let _ = ::icarus_runtime::ToolRegistry::register_sync_executor(
                    tool_id,
                    #executor_fn_name
                );
            };
        }
    }
}

/// Extracts documentation comment from function attributes.
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_parts = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    let content = lit_str.value();
                    doc_parts.push(content.trim().to_string());
                }
            }
        }
    }

    if doc_parts.is_empty() {
        None
    } else {
        Some(doc_parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comment() {
        let input: ItemFn = syn::parse_quote! {
            /// This is a test function
            /// that does something useful.
            fn test_fn() {}
        };

        let doc = extract_doc_comment(&input.attrs);
        assert_eq!(
            doc,
            Some("This is a test function that does something useful.".to_string())
        );
    }

    #[test]
    fn test_validate_function_signature() {
        // Valid function
        let valid_fn: ItemFn = syn::parse_quote! {
            fn valid_tool(x: i32) -> String { "test".to_string() }
        };
        assert!(validate_function_signature(&valid_fn).is_ok());

        // Invalid function with self parameter
        let invalid_fn: ItemFn = syn::parse_quote! {
            fn invalid_tool(&self, x: i32) -> String { "test".to_string() }
        };
        assert!(validate_function_signature(&invalid_fn).is_err());

        // Invalid function with generics
        let generic_fn: ItemFn = syn::parse_quote! {
            fn generic_tool<T>(x: T) -> String { "test".to_string() }
        };
        assert!(validate_function_signature(&generic_fn).is_err());
    }

    #[test]
    fn test_parameter_count_limit() {
        // Create a function with exactly 50 parameters (should pass)
        let params_50 = (0..50)
            .map(|i| format!("p{}: i32", i))
            .collect::<Vec<_>>()
            .join(", ");
        let fn_with_50 = syn::parse_str::<ItemFn>(&format!(
            "fn tool_50({}) -> String {{ \"test\".to_string() }}",
            params_50
        ))
        .unwrap();
        let result_50 = tool_impl(TokenStream::new(), quote::quote! { #fn_with_50 });
        assert!(result_50.is_ok(), "50 parameters should be allowed");

        // Create a function with 51 parameters (should fail)
        let params_51 = (0..51)
            .map(|i| format!("p{}: i32", i))
            .collect::<Vec<_>>()
            .join(", ");
        let fn_with_51 = syn::parse_str::<ItemFn>(&format!(
            "fn tool_51({}) -> String {{ \"test\".to_string() }}",
            params_51
        ))
        .unwrap();
        let result_51 = tool_impl(TokenStream::new(), quote::quote! { #fn_with_51 });
        assert!(result_51.is_err(), "51 parameters should be rejected");

        if let Err(e) = result_51 {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("cannot have more than 50 parameters"),
                "Error message should mention the parameter limit, got: {}",
                error_msg
            );
        }
    }
}
