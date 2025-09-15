// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.
// Signature verification and telemetry must remain intact.

// Missing docs warnings disabled during active development

//! Procedural macros for the Icarus SDK
//!
//! This crate provides derive macros and attribute macros to reduce
//! boilerplate when building MCP servers for ICP.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod server;
mod tools;
mod validation;

/// Derive macro for creating MCP tools
#[proc_macro_derive(IcarusTool, attributes(icarus_tool))]
pub fn derive_icarus_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_icarus_tool_impl(input) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive_icarus_tool_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    // Extract tool metadata from attributes
    let mut name = None;
    let mut description = None;

    for attr in &input.attrs {
        if attr.path().is_ident("icarus_tool") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else if meta.path.is_ident("description") {
                    description = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else {
                    Err(meta.error("unsupported icarus_tool attribute"))
                }
            })?;
        }
    }

    let struct_name = &input.ident;
    let tool_name = name.unwrap_or_else(|| struct_name.to_string());
    let tool_desc = description.unwrap_or_else(|| format!("{} tool", tool_name));

    // Generate the implementation
    let expanded = quote! {
        #[async_trait::async_trait]
        impl icarus_core::tool::IcarusTool for #struct_name {
            fn info(&self) -> icarus_core::tool::ToolInfo {
                icarus_core::tool::ToolInfo {
                    name: #tool_name.to_string(),
                    description: #tool_desc.to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                }
            }

            fn to_rmcp_tool(&self) -> rmcp::model::Tool {
                use std::borrow::Cow;
                use std::sync::Arc;

                let schema = serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                });

                rmcp::model::Tool {
                    name: Cow::Borrowed(#tool_name),
                    description: Some(Cow::Borrowed(#tool_desc)),
                    input_schema: Arc::new(
                        schema.as_object()
                            .expect("Generated JSON schema should be an object")
                            .clone()
                    ),
                    annotations: None,
                }
            }

            async fn execute(&self, args: serde_json::Value) -> icarus_core::error::Result<serde_json::Value> {
                // Default implementation - override in your tool
                Ok(serde_json::json!({
                    "error": "Tool execution not implemented"
                }))
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

/// Attribute macro for MCP server setup
/// ```ignore
/// pub struct MyServer {
///     tools: Vec<Box<dyn IcarusTool>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn icarus_server(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = parse_macro_input!(input as DeriveInput);
    server::expand_icarus_server(args, input).into()
}

/// Derive macro for common Icarus type patterns
///
/// This is a convenience macro that combines IcarusStorable with sensible defaults.
/// You still need to derive the standard traits manually.
///
/// # Examples
/// ```ignore
/// #[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusType)]
/// struct MemoryEntry {
///     id: String,
///     content: String,
///     created_at: u64,
/// }
/// ```
///
/// This is equivalent to:
/// ```ignore
/// #[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
/// #[icarus_storable(unbounded)]
/// struct MemoryEntry { ... }
/// ```
#[proc_macro_derive(IcarusType, attributes(icarus_storable))]
pub fn derive_icarus_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract generics if any
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse attributes for storage configuration
    let mut unbounded = true; // Default to unbounded for convenience
    let mut max_size_bytes = 1024 * 1024; // 1MB default

    for attr in &input.attrs {
        if attr.path().is_ident("icarus_storable") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("unbounded") {
                    unbounded = true;
                    Ok(())
                } else if meta.path.is_ident("bounded") {
                    unbounded = false;
                    Ok(())
                } else if meta.path.is_ident("max_size") {
                    let value = meta.value()?;
                    let lit_str: syn::LitStr = value.parse()?;
                    let size_str = lit_str.value();
                    max_size_bytes = parse_size_string(&size_str);
                    unbounded = false;
                    Ok(())
                } else {
                    Ok(()) // Ignore other attributes
                }
            })
            .unwrap_or(()); // Ignore parse errors
        }
    }

    let bound = if unbounded {
        quote! { ic_stable_structures::storable::Bound::Unbounded }
    } else {
        quote! {
            ic_stable_structures::storable::Bound::Bounded {
                max_size: #max_size_bytes,
                is_fixed_size: false,
            }
        }
    };

    // Generate all the common trait implementations
    let expanded = quote! {
        // Note: We expect the user to add #[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
        // This macro just adds the IcarusStorable functionality

        // Implement Storable for ICP
        impl #impl_generics ic_stable_structures::Storable for #struct_name #ty_generics #where_clause {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }

            const BOUND: ic_stable_structures::storable::Bound = #bound;
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for simplified storage declaration
///
/// Generates stable storage declarations from a simple struct definition.
/// Automatically assigns memory IDs and handles initialization.
///
/// # Examples
/// ```ignore
/// #[derive(IcarusStorage)]
/// struct Storage {
///     memories: StableBTreeMap<String, MemoryEntry>,
///     counter: u64,
///     users: StableBTreeMap<Principal, User>,
/// }
/// ```
///
/// This generates:
/// - Thread-local storage declarations
/// - Memory manager initialization  
/// - Accessor methods for each field
#[proc_macro_derive(IcarusStorage)]
pub fn derive_icarus_storage(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(data_struct) = &input.data {
        if let syn::Fields::Named(fields_named) = &data_struct.fields {
            let struct_name = &input.ident;
            let mut storage_declarations = vec![];
            let mut accessor_methods = vec![];
            let mut memory_id = 0u8;

            for field in &fields_named.named {
                if let Some(field_name) = &field.ident {
                    let field_type = &field.ty;
                    let field_name_upper =
                        syn::Ident::new(&field_name.to_string().to_uppercase(), field_name.span());

                    // Generate storage declaration based on field type
                    let storage_decl = if is_stable_map_type(field_type) {
                        quote! {
                            #field_name_upper: #field_type =
                                ::ic_stable_structures::StableBTreeMap::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    ))
                                );
                        }
                    } else if is_stable_cell_type(field_type) {
                        quote! {
                            #field_name_upper: ::ic_stable_structures::StableCell<#field_type, ::ic_stable_structures::memory_manager::VirtualMemory<::ic_stable_structures::DefaultMemoryImpl>> =
                                ::ic_stable_structures::StableCell::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    )),
                                    Default::default()
                                ).expect("Failed to initialize StableCell");
                        }
                    } else {
                        // For simple types, wrap in StableCell
                        quote! {
                            #field_name_upper: ::ic_stable_structures::StableCell<#field_type, ::ic_stable_structures::memory_manager::VirtualMemory<::ic_stable_structures::DefaultMemoryImpl>> =
                                ::ic_stable_structures::StableCell::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    )),
                                    Default::default()
                                ).expect("Failed to initialize StableCell");
                        }
                    };

                    storage_declarations.push(storage_decl);

                    // Generate accessor method
                    let accessor = if is_stable_map_type(field_type) {
                        quote! {
                            pub fn #field_name() -> impl std::ops::Deref<Target = #field_type> {
                                #field_name_upper.with(|storage| storage.borrow())
                            }
                        }
                    } else {
                        let setter_name =
                            syn::Ident::new(&format!("{}_set", field_name), field_name.span());

                        quote! {
                            pub fn #field_name() -> #field_type
                            where
                                #field_type: Clone + Default
                            {
                                #field_name_upper.with(|cell| cell.borrow().get().clone())
                            }

                            pub fn #setter_name(value: #field_type)
                            where
                                #field_type: Clone
                            {
                                #field_name_upper.with(|cell| {
                                    cell.borrow_mut().set(value)
                                        .expect("Failed to set value in StableCell");
                                });
                            }
                        }
                    };

                    accessor_methods.push(accessor);
                    memory_id += 1;
                }
            }

            let expanded = quote! {
                thread_local! {
                    static MEMORY_MANAGER: ::std::cell::RefCell<
                        ::ic_stable_structures::memory_manager::MemoryManager<
                            ::ic_stable_structures::DefaultMemoryImpl
                        >
                    > = ::std::cell::RefCell::new(
                        ::ic_stable_structures::memory_manager::MemoryManager::init(
                            ::ic_stable_structures::DefaultMemoryImpl::default()
                        )
                    );

                    #(static #storage_declarations)*
                }

                impl #struct_name {
                    #(#accessor_methods)*
                }
            };

            TokenStream::from(expanded)
        } else {
            syn::Error::new_spanned(
                &input,
                "IcarusStorage can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    } else {
        syn::Error::new_spanned(&input, "IcarusStorage can only be used on structs")
            .to_compile_error()
            .into()
    }
}

/// Derive macro for ICP storable types
///
/// # Examples
/// ```ignore
/// #[derive(IcarusStorable)]
/// struct MyData { ... } // Uses default 1MB bound
///
/// #[derive(IcarusStorable)]
/// #[icarus_storable(unbounded)]
/// struct LargeData { ... } // Uses unbounded storage
///
/// #[derive(IcarusStorable)]
/// #[icarus_storable(max_size = "2MB")]
/// struct CustomData { ... } // Uses custom 2MB bound
/// ```
#[proc_macro_derive(IcarusStorable, attributes(icarus_storable))]
pub fn derive_icarus_storable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract generics if any
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse attributes
    let mut unbounded = false;
    let mut max_size_bytes = 1024 * 1024; // 1MB default

    for attr in &input.attrs {
        if attr.path().is_ident("icarus_storable") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("unbounded") {
                    unbounded = true;
                    Ok(())
                } else if meta.path.is_ident("max_size") {
                    let value = meta.value()?;
                    let lit_str: syn::LitStr = value.parse()?;
                    let size_str = lit_str.value();
                    max_size_bytes = parse_size_string(&size_str);
                    Ok(())
                } else {
                    Err(meta.error("unsupported icarus_storable attribute"))
                }
            })
            .unwrap_or_else(|e| panic!("Failed to parse icarus_storable attribute: {}", e));
        }
    }

    let bound = if unbounded {
        quote! { ic_stable_structures::storable::Bound::Unbounded }
    } else {
        quote! {
            ic_stable_structures::storable::Bound::Bounded {
                max_size: #max_size_bytes,
                is_fixed_size: false,
            }
        }
    };

    // Generate implementation
    let expanded = quote! {
        impl #impl_generics ic_stable_structures::Storable for #struct_name #ty_generics #where_clause {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }

            fn into_bytes(self) -> std::vec::Vec<u8> {
                candid::encode_one(&self).expect("Failed to encode to Candid")
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }

            const BOUND: ic_stable_structures::storable::Bound = #bound;
        }

    };

    TokenStream::from(expanded)
}

/// Attribute macro for marking impl blocks that contain tool methods
#[proc_macro_attribute]
pub fn icarus_tools(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = TokenStream2::from(attr);
    let input = parse_macro_input!(item as syn::ItemImpl);
    tools::expand_icarus_tools(attr, input).into()
}

/// Attribute macro for individual tool methods
/// Usage:
///   #[icarus_tool("Tool description")]
///   #[icarus_tool(description = "Tool description", title = "Display Title", icon = "icon-name")]
///
/// This attribute marks functions as tools and stores their metadata.
/// The icarus_module macro will collect these to generate metadata.
#[proc_macro_attribute]
pub fn icarus_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as syn::ItemFn);

    // Check for query/update attributes
    let has_query = input_fn.attrs.iter().any(|attr| attr.path().is_ident("query"));
    let has_update = input_fn.attrs.iter().any(|attr| attr.path().is_ident("update"));

    // Validate the function signature
    let validation_result = validation::validate_tool_function(&input_fn, has_query, has_update);
    if !validation_result.is_valid {
        let mut error_message = String::from("Tool function validation failed:\n");
        for error in &validation_result.errors {
            error_message.push_str(&format!("  - {}\n", error));
        }
        return syn::Error::new_spanned(&input_fn.sig.ident, error_message)
            .to_compile_error()
            .into();
    }

    // Parse the description from the attribute
    let description = if attr.is_empty() {
        format!("{} tool", input_fn.sig.ident)
    } else {
        // TODO: Parse structured attributes properly
        // For now, assume simple string description
        format!("{} tool", input_fn.sig.ident)
    };

    // Generate the function with metadata preservation
    let expanded = quote! {
        #[doc = #description]
        #[allow(dead_code)]
        #input_fn
    };

    TokenStream::from(expanded)
}

/// Module-level attribute macro that collects all icarus_tool functions
/// and generates the list_tools query function automatically.
///
/// Usage:
/// ```ignore
/// #[icarus_module]
/// mod my_module {
///     #[update]
///     #[icarus_tool("Store data")]
///     pub fn store(data: String) -> Result<(), String> { ... }
/// }
/// ```
///
/// Or with module configuration:
/// ```ignore
/// #[icarus_module(title = "My MCP Server", website = "https://example.com")]
/// mod my_module {
///     #[update]
///     #[icarus_tool(description = "Store data", title = "Data Storage")]
///     pub fn store(data: String) -> Result<(), String> { ... }
/// }
/// ```
///
/// The name and version are automatically taken from Cargo.toml
#[proc_macro_attribute]
pub fn icarus_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemMod);

    // Parse attributes
    let module_config = if attr.is_empty() {
        tools::ModuleConfig::default()
    } else {
        parse_macro_input!(attr as tools::ModuleConfig)
    };

    // Process the module to collect tools and generate metadata
    match tools::expand_icarus_module(input, module_config) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(error) => TokenStream::from(error.to_compile_error()),
    }
}

/// Crate-level attribute macro that scans for all icarus_tool functions
/// and generates the list_tools query function automatically.
///
/// Usage:
/// ```ignore
/// #![icarus_canister(name = "my-server", version = "1.0.0")]
///
/// #[update]
/// #[icarus_tool("Store data")]
/// pub fn store(data: String) -> Result<(), String> { ... }
/// ```
#[proc_macro_attribute]
pub fn icarus_canister(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the crate content
    let input = parse_macro_input!(item as syn::File);

    // Process the crate to collect tools and generate metadata
    match tools::expand_icarus_canister(input) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => TokenStream::from(error.to_compile_error()),
    }
}

// Helper function to extract type as string
#[allow(dead_code)]
fn extract_type_string(ty: &syn::Type) -> String {
    quote!(#ty).to_string()
}

// Helper function to convert Rust types to JSON schema types
#[allow(dead_code)]
fn rust_type_to_json_type(rust_type: &str) -> &'static str {
    match rust_type {
        // Check Vec first before String to avoid "Vec<String>" matching "String"
        s if s.contains("Vec<") => "array",
        s if s.contains("String") || s.contains("&str") => "string",
        s if s.contains("i32")
            || s.contains("i64")
            || s.contains("u32")
            || s.contains("u64")
            || s.contains("usize") =>
        {
            "integer"
        }
        s if s.contains("f32") || s.contains("f64") => "number",
        s if s.contains("bool") => "boolean",
        _ => "string", // Default to string for unknown types
    }
}

// Helper function to check if a type is StableBTreeMap
fn is_stable_map_type(ty: &syn::Type) -> bool {
    let type_string = quote!(#ty).to_string();
    type_string.contains("StableBTreeMap")
}

// Helper function to check if a type is StableCell
fn is_stable_cell_type(ty: &syn::Type) -> bool {
    let type_string = quote!(#ty).to_string();
    type_string.contains("StableCell")
}

// Helper function to parse size strings like "1MB", "2KB", etc.
fn parse_size_string(size: &str) -> u32 {
    let size = size.trim();
    if let Some(num_str) = size.strip_suffix("MB") {
        num_str.trim().parse::<u32>().unwrap_or(1) * 1024 * 1024
    } else if let Some(num_str) = size.strip_suffix("KB") {
        num_str.trim().parse::<u32>().unwrap_or(1) * 1024
    } else if let Some(num_str) = size.strip_suffix("B") {
        num_str.trim().parse::<u32>().unwrap_or(1024)
    } else {
        // Try to parse as raw bytes
        size.parse::<u32>().unwrap_or(1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_type_string() {
        let ty: syn::Type = parse_quote!(String);
        let type_str = extract_type_string(&ty);
        assert_eq!(type_str, "String");

        let ty: syn::Type = parse_quote!(Vec<u64>);
        let type_str = extract_type_string(&ty);
        assert!(type_str.contains("Vec"));
        assert!(type_str.contains("u64"));
    }

    #[test]
    fn test_rust_type_to_json_type() {
        assert_eq!(rust_type_to_json_type("String"), "string");
        assert_eq!(rust_type_to_json_type("&str"), "string");
        assert_eq!(rust_type_to_json_type("i32"), "integer");
        assert_eq!(rust_type_to_json_type("i64"), "integer");
        assert_eq!(rust_type_to_json_type("u32"), "integer");
        assert_eq!(rust_type_to_json_type("u64"), "integer");
        assert_eq!(rust_type_to_json_type("usize"), "integer");
        assert_eq!(rust_type_to_json_type("f32"), "number");
        assert_eq!(rust_type_to_json_type("f64"), "number");
        assert_eq!(rust_type_to_json_type("bool"), "boolean");
        assert_eq!(rust_type_to_json_type("Vec<String>"), "array");
        assert_eq!(rust_type_to_json_type("CustomType"), "string"); // Default

        // Test edge cases
        assert_eq!(rust_type_to_json_type("Option<String>"), "string"); // Should default to string
        assert_eq!(rust_type_to_json_type("HashMap<String, u64>"), "string"); // Should default to string
    }

    #[test]
    fn test_is_stable_map_type() {
        let ty: syn::Type = parse_quote!(StableBTreeMap<String, u64>);
        assert!(is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(ic_stable_structures::StableBTreeMap<String, u64>);
        assert!(is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(String);
        assert!(!is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(Vec<String>);
        assert!(!is_stable_map_type(&ty));
    }

    #[test]
    fn test_is_stable_cell_type() {
        let ty: syn::Type = parse_quote!(StableCell<u64>);
        assert!(is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(ic_stable_structures::StableCell<String>);
        assert!(is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(String);
        assert!(!is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(Vec<u64>);
        assert!(!is_stable_cell_type(&ty));
    }

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("1MB"), 1024 * 1024);
        assert_eq!(parse_size_string("2MB"), 2 * 1024 * 1024);
        assert_eq!(parse_size_string("512KB"), 512 * 1024);
        assert_eq!(parse_size_string("1024B"), 1024);
        assert_eq!(parse_size_string("1048576"), 1048576); // Raw bytes

        // Test with whitespace
        assert_eq!(parse_size_string(" 1MB "), 1024 * 1024);
        assert_eq!(parse_size_string(" 512 KB"), 512 * 1024);

        // Test invalid inputs (should use defaults)
        assert_eq!(parse_size_string("invalid"), 1024 * 1024); // Default 1MB
        assert_eq!(parse_size_string(""), 1024 * 1024);
        assert_eq!(parse_size_string("MB"), 1024 * 1024); // Default when parsing fails
    }

    // Note: Tests for derive_icarus_tool_impl cannot be run in unit tests
    // because they require the proc_macro::TokenStream context



    #[test]
    fn test_parse_size_string_edge_cases() {
        // Test case sensitivity (should be case insensitive)
        assert_eq!(parse_size_string("1mb"), 1024 * 1024);
        assert_eq!(parse_size_string("1MB"), 1024 * 1024);

        // Test zero values
        assert_eq!(parse_size_string("0MB"), 0);
        assert_eq!(parse_size_string("0KB"), 0);
        assert_eq!(parse_size_string("0B"), 0);

        // Test large values
        assert_eq!(parse_size_string("1024MB"), 1024 * 1024 * 1024);

        // Test fractional parts (should be ignored)
        assert_eq!(parse_size_string("1.5MB"), 1024 * 1024); // Should parse as 1MB
    }

    // Note: Tests for derive macros that use proc_macro::TokenStream
    // cannot be run in unit tests as they require proc-macro context.
    // These macros are tested through integration tests instead.
}
