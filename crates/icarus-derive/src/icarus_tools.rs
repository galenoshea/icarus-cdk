//! #[icarus_tools] macro implementation for trait-based tool providers
//!
//! This macro generates IC CDK wrappers, service instances, and MCP metadata
//! from trait implementations marked with #[icarus_tools].

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Lit, Meta, Type};

// Configuration could be added here in the future if needed

/// Information about a tool method extracted from the trait impl
#[derive(Debug, Clone)]
pub struct ToolMethodInfo {
    pub method_name: Ident,
    pub description: String,
    pub is_query: bool,
    pub requires_auth: bool,
    pub skip_auth: bool,
    pub title: Option<String>,
}

/// Parse attributes from a method to extract tool metadata
fn parse_method_attributes(method: &ImplItemFn) -> Option<ToolMethodInfo> {
    let mut description = None;
    let mut is_query = false;
    let mut requires_auth = true;
    let mut skip_auth = false;
    let mut title = None;
    let mut is_tool = false;

    // Parse all attributes
    for attr in &method.attrs {
        match attr.meta {
            Meta::Path(ref path) if path.is_ident("query") => {
                is_query = true;
            }
            Meta::Path(ref path) if path.is_ident("update") => {
                is_query = false;
            }
            Meta::Path(ref path) if path.is_ident("skip_auth") => {
                skip_auth = true;
                requires_auth = false;
            }
            Meta::List(ref meta_list) if meta_list.path.is_ident("tool") => {
                is_tool = true;
                // Parse tool attribute content
                if let Ok(Lit::Str(lit_str)) = meta_list.parse_args::<Lit>() {
                    description = Some(lit_str.value());
                }
            }
            Meta::NameValue(ref meta_name_value) => {
                if meta_name_value.path.is_ident("title") {
                    if let Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            title = Some(lit_str.value());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Only process methods marked with #[tool]
    if !is_tool {
        return None;
    }

    Some(ToolMethodInfo {
        method_name: method.sig.ident.clone(),
        description: description.unwrap_or_else(|| format!("{} function", method.sig.ident)),
        is_query,
        requires_auth,
        skip_auth,
        title,
    })
}

/// Generate standalone tool methods (not part of trait)
fn generate_tool_methods(
    tools: &[ToolMethodInfo],
    impl_block: &ItemImpl,
    service_type: &Type,
) -> Vec<TokenStream> {
    tools
        .iter()
        .map(|tool| {
            let method_name = &tool.method_name;

            // Find the original method in the impl block
            let original_method = impl_block
                .items
                .iter()
                .find_map(|item| {
                    if let ImplItem::Fn(method) = item {
                        if method.sig.ident == *method_name {
                            Some(method)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .expect("Tool method should exist in impl block");

            // Create a standalone version without IC CDK attributes
            let mut standalone_method = original_method.clone();

            // Remove IC CDK attributes (#[update], #[query]) from the standalone method
            standalone_method.attrs.retain(|attr| {
                !matches!(attr.meta,
                    Meta::Path(ref path) if path.is_ident("update") || path.is_ident("query")
                )
            });

            // Make it a static method by removing &self parameter if present
            let mut new_inputs = syn::punctuated::Punctuated::new();
            for input in &standalone_method.sig.inputs {
                if !matches!(input, FnArg::Receiver(_)) {
                    new_inputs.push(input.clone());
                }
            }
            standalone_method.sig.inputs = new_inputs;

            quote! {
                impl #service_type {
                    #standalone_method
                }
            }
        })
        .collect()
}

/// Generate a static service instance
fn generate_service_instance(_service_type: &Type) -> TokenStream {
    // For ZST services, we don't need a static instance since we can just call methods directly
    // This generates zero-cost static dispatch
    quote! {
        // Zero-cost static dispatch - no instance needed for ZST services
        // Methods are called directly on the service type
    }
}

/// Generate IC CDK wrapper function for a tool method
fn generate_ic_wrapper(tool_info: &ToolMethodInfo, service_type: &Type) -> TokenStream {
    let method_name = &tool_info.method_name;

    let ic_attr = if tool_info.is_query {
        quote! { #[ic_cdk_macros::query] }
    } else {
        quote! { #[ic_cdk_macros::update] }
    };

    let candid_attr = if tool_info.is_query {
        quote! { #[candid::candid_method(query)] }
    } else {
        quote! { #[candid::candid_method(update)] }
    };

    let auth_check = if tool_info.requires_auth && !tool_info.skip_auth {
        quote! {
            // Check authentication (authenticate() traps on failure)
            let _auth_info = ::icarus::canister::auth::authenticate();
        }
    } else {
        quote! {}
    };

    quote! {
        #ic_attr
        #candid_attr
        pub async fn #method_name(input: String) -> String {
            #auth_check

            // Parse MCP JSON input to typed parameter
            let args = match serde_json::from_str(&input) {
                Ok(args) => args,
                Err(e) => {
                    ic_cdk::trap(&format!("Invalid JSON input for {}: {}", stringify!(#method_name), e));
                }
            };

            // Call static service method
            match #service_type::#method_name(args).await {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => json,
                        Err(e) => {
                            ic_cdk::trap(&format!("Failed to serialize result from {}: {}", stringify!(#method_name), e));
                        }
                    }
                }
                Err(e) => {
                    ic_cdk::trap(&e);
                }
            }
        }
    }
}

/// Generate the list_tools function for MCP discovery
fn generate_list_tools_function(tools: &[ToolMethodInfo]) -> TokenStream {
    let tool_entries: Vec<TokenStream> = tools
        .iter()
        .map(|tool| {
            let name = tool.method_name.to_string();
            let description = &tool.description;
            let title = tool.title.as_ref().unwrap_or(&name);

            quote! {
                serde_json::json!({
                    "name": #name,
                    "description": #description,
                    "title": #title,
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "input": {
                                "type": "string",
                                "description": "JSON input for the tool"
                            }
                        },
                        "required": ["input"]
                    }
                })
            }
        })
        .collect();

    quote! {
        #[ic_cdk_macros::query]
        #[candid::candid_method(query)]
        pub fn list_tools() -> String {
            let tools = vec![
                #(#tool_entries),*
            ];

            serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "tools": tools
            }).to_string()
        }
    }
}

/// Generate authentication and lifecycle functions
fn generate_boilerplate_functions(macro_attrs: &MacroAttributes) -> TokenStream {
    // Generate extension initialization code if extensions are specified
    let extension_init = if !macro_attrs.extensions.is_empty() {
        let extension_inits: Vec<TokenStream> = macro_attrs
            .extensions
            .iter()
            .map(|ext_path| {
                let ext_name = ext_path.segments.last().unwrap().ident.to_string();
                quote! {
                    // Initialize #ext_path extension
                    match <#ext_path as InitializationExtension>::initialize(
                        <#ext_path as InitializationExtension>::Config::default()
                    ) {
                        Ok(_) => {},
                        Err(e) => ic_cdk::trap(&format!("Failed to initialize {} extension: {}", #ext_name, e)),
                    }
                }
            })
            .collect();

        quote! {
            // Initialize extensions
            #(#extension_inits)*
        }
    } else {
        quote! {
            // No extensions to initialize
        }
    };

    quote! {
        #[ic_cdk_macros::init]
        pub fn init(owner: candid::Principal) {
            // Initialize authentication first (marketplace requirement)
            ::icarus::canister::auth::init_auth(owner);

            // Initialize extensions if any
            #extension_init
        }

        #[ic_cdk_macros::post_upgrade]
        pub fn post_upgrade() {
            // Stable memory is automatically restored
            // Extensions handle their own upgrade logic if needed
        }

        #[ic_cdk_macros::query]
        #[candid::candid_method(query)]
        pub fn get_auth_status() -> String {
            serde_json::to_string(&::icarus::canister::auth::get_auth_status())
                .unwrap_or_else(|_| "{}".to_string())
        }

        #[ic_cdk_macros::update]
        #[candid::candid_method(update)]
        pub fn add_user(principal: candid::Principal, role: String) -> Result<String, String> {
            let auth_role = match role.as_str() {
                "owner" => ::icarus::canister::auth::AuthRole::Owner,
                "admin" => ::icarus::canister::auth::AuthRole::Admin,
                "user" => ::icarus::canister::auth::AuthRole::User,
                _ => return Err(format!("Invalid role: {}", role)),
            };
            Ok(::icarus::canister::auth::add_user(principal, auth_role))
        }
    }
}

/// Information about macro attributes
#[derive(Debug, Clone, Default)]
pub struct MacroAttributes {
    pub extensions: Vec<syn::Path>,
}

/// Parse macro attributes from the attribute list
fn parse_macro_attributes(attrs: &[syn::Attribute]) -> syn::Result<MacroAttributes> {
    let mut result = MacroAttributes::default();

    for attr in attrs {
        // Since we're creating synthetic attributes, we need to check the meta directly
        match &attr.meta {
            syn::Meta::List(meta_list) if meta_list.path.is_ident("icarus_tools") => {
                // Parse the attribute arguments
                for nested in meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )? {
                    if let syn::Meta::NameValue(nv) = nested {
                        if nv.path.is_ident("extensions") {
                            if let syn::Expr::Array(array) = &nv.value {
                                for elem in &array.elems {
                                    if let syn::Expr::Path(path_expr) = elem {
                                        result.extensions.push(path_expr.path.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("extensions") => {
                // Handle direct extensions attribute
                if let syn::Expr::Array(array) = &nv.value {
                    for elem in &array.elems {
                        if let syn::Expr::Path(path_expr) = elem {
                            result.extensions.push(path_expr.path.clone());
                        }
                    }
                }
            }
            _ => {
                // Handle the case where there are no arguments: #[icarus_tools]
                // This is backward compatible - no extensions will be initialized
            }
        }
    }

    Ok(result)
}

/// Main macro implementation for #[icarus_tools]
pub fn expand_icarus_tools(
    attrs: Vec<syn::Attribute>,
    input: ItemImpl,
) -> syn::Result<TokenStream> {
    // Parse macro attributes
    let macro_attrs = parse_macro_attributes(&attrs)?;

    // Validate this is a trait implementation
    let _trait_path = input.trait_.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "icarus_tools can only be applied to trait implementations",
        )
    })?;

    // Extract service type
    let service_type = &input.self_ty;

    // Process all methods to find tool methods
    let mut tool_methods = Vec::new();
    let mut clean_impl_items = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            if let Some(tool_info) = parse_method_attributes(method) {
                tool_methods.push(tool_info);
                // Don't include tool methods in the clean trait implementation
            } else {
                // Keep non-tool methods in the trait implementation
                clean_impl_items.push(item.clone());
            }
        } else {
            // Keep all non-function items
            clean_impl_items.push(item.clone());
        }
    }

    if tool_methods.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "No methods marked with #[tool] found in trait implementation",
        ));
    }

    // Create clean trait implementation without tool methods
    let mut clean_impl = input.clone();
    clean_impl.items = clean_impl_items;

    // Generate components
    let service_instance = generate_service_instance(service_type);
    let tool_method_impls = generate_tool_methods(&tool_methods, &input, service_type);
    let ic_wrappers: Vec<TokenStream> = tool_methods
        .iter()
        .map(|tool| generate_ic_wrapper(tool, service_type))
        .collect();
    let list_tools_fn = generate_list_tools_function(&tool_methods);
    let boilerplate_fns = generate_boilerplate_functions(&macro_attrs);

    Ok(quote! {
        // Clean trait implementation (without tool methods)
        #clean_impl

        // Generated service instance
        #service_instance

        // Generated standalone tool method implementations
        #(#tool_method_impls)*

        // Generated IC CDK wrappers
        #(#ic_wrappers)*

        // Generated MCP metadata function
        #list_tools_fn

        // Generated boilerplate functions
        #boilerplate_fns

        // Note: Candid interface is extracted via candid-extractor tool during build
        // instead of using ic_cdk::export_candid!() which is incompatible with WASI
    })
}
