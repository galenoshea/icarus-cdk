//! Tools macro implementation
//!
//! Generates Candid methods from tool definitions

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse::ParseStream, File, FnArg, Item, ItemFn, ItemMod, Pat};

use crate::parse_tool_metadata;

fn extract_tool_description(attr: &syn::Attribute) -> Option<String> {
    // Parse #[icarus_tool("description")] using enhanced metadata parsing
    if attr.path().is_ident("icarus_tool") {
        let metadata = parse_tool_metadata(attr.meta.to_token_stream());
        if !metadata.description.is_empty() {
            return Some(metadata.description);
        }
    }
    None
}

fn extract_tool_title(attr: &syn::Attribute) -> Option<String> {
    // Parse #[icarus_tool] to extract title parameter using enhanced metadata parsing
    if attr.path().is_ident("icarus_tool") {
        let metadata = parse_tool_metadata(attr.meta.to_token_stream());
        metadata.title
    } else {
        None
    }
}

fn extract_tool_icon(attr: &syn::Attribute) -> Option<String> {
    // Parse #[icarus_tool] to extract icon parameter
    if attr.path().is_ident("icarus_tool") {
        let mut icon = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("icon") {
                if let Ok(value) = meta.value() {
                    if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                        icon = Some(lit_str.value());
                    }
                }
            }
            Ok(())
        });
        icon
    } else {
        None
    }
}

/// Configuration for icarus_module attribute
/// Currently empty as authentication is always mandatory
#[derive(Debug, Clone, Default)]
pub struct ModuleConfig {
    /// Optional display title for the module
    pub title: Option<String>,
    /// Optional website URL for the module
    pub website_url: Option<String>,
}

impl Parse for ModuleConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = ModuleConfig::default();

        // If no parameters, return default
        if input.is_empty() {
            return Ok(config);
        }

        // Parse named parameters
        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let value: syn::LitStr = input.parse()?;

            match name.to_string().as_str() {
                "title" => config.title = Some(value.value()),
                "website" => config.website_url = Some(value.value()),
                "website_url" => config.website_url = Some(value.value()),
                _ => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!(
                            "Unknown parameter '{}'. Supported parameters: title, website",
                            name
                        ),
                    ))
                }
            }

            // Check for comma (optional for last parameter)
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(config)
    }
}

/// Processed function attributes result
#[derive(Debug)]
struct ProcessedAttributes {
    has_update: bool,
    has_query: bool,
    skip_auth: bool,
    has_require_role: bool,
    description: Option<String>,
    title: Option<String>,
    icon: Option<String>,
}

impl ProcessedAttributes {
    fn new() -> Self {
        Self {
            has_update: false,
            has_query: false,
            skip_auth: false,
            has_require_role: false,
            description: None,
            title: None,
            icon: None,
        }
    }

    fn has_canister_attribute(&self) -> bool {
        self.has_update || self.has_query
    }

    fn needs_authentication_injection(&self) -> bool {
        !self.skip_auth && !self.has_require_role
    }
}

/// Extract and process all attributes from a function in a single optimized pass
fn process_function_attributes(func: &ItemFn) -> ProcessedAttributes {
    let mut attrs = ProcessedAttributes::new();

    for attr in &func.attrs {
        if attr.path().is_ident("update") {
            attrs.has_update = true;
        } else if attr.path().is_ident("query") {
            attrs.has_query = true;
        } else if attr.path().is_ident("skip_auth") {
            attrs.skip_auth = true;
        } else if attr.path().is_ident("require_role") {
            attrs.has_require_role = true;
        } else if attr.path().is_ident("doc") && attrs.description.is_none() {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                attrs.description = Some(lit.value());
            }
        } else if let Some(desc) = extract_tool_description(attr) {
            attrs.description = Some(desc);
        } else if let Some(t) = extract_tool_title(attr) {
            attrs.title = Some(t);
        } else if let Some(i) = extract_tool_icon(attr) {
            attrs.icon = Some(i);
        }
    }

    attrs
}

/// Extract function parameters in optimized format
fn extract_function_parameters(func: &ItemFn) -> Vec<(syn::Ident, Box<syn::Type>)> {
    func.sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(ident) = &*pat_type.pat {
                    Some((ident.ident.clone(), pat_type.ty.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Inject authenticate() call at the beginning of a function
/// This should only be called for functions that don't already have require_role attributes
fn inject_authenticate_call(func: &mut ItemFn) {
    use syn::{parse_quote, Stmt};

    // Default to basic authenticate() call
    let auth_call: Stmt = parse_quote! {
        ::icarus::canister::auth::authenticate();
    };

    // Insert at the beginning of the function body
    func.block.stmts.insert(0, auth_call);
}

/// Generate the init function for canister initialization
fn generate_init_function() -> TokenStream {
    quote! {
        /// Canister initialization function (auto-generated by icarus_module)
        /// Requires an owner principal for deployment
        #[::ic_cdk_macros::init]
        fn init(owner: ::candid::Principal) {
            // Initialize the authentication system with the provided owner
            ::icarus::canister::auth::init_auth(owner);

            // Log initialization
            ::ic_cdk::api::debug_print(format!(
                "{} canister initialized with owner: {}",
                env!("CARGO_PKG_NAME"),
                owner
            ));
        }

        /// Post-upgrade hook to maintain state (auto-generated by icarus_module)
        #[::ic_cdk_macros::post_upgrade]
        fn post_upgrade() {
            // State is preserved in stable memory, no action needed
        }
    }
}

/// Generate authentication management functions
fn generate_auth_functions() -> TokenStream {
    quote! {
        /// Add a user to the authorized users list
        /// Requires Admin role or higher
        #[::ic_cdk_macros::update]
        pub fn add_authorized_user(principal_text: String, role: String) -> Result<String, String> {
            use ::icarus::canister::auth::{add_user, AuthRole, require_role_or_higher};
            use ::candid::Principal;

            // Require Admin or Owner role
            require_role_or_higher(AuthRole::Admin);

            // Parse the principal
            let principal = Principal::from_text(principal_text)
                .map_err(|e| format!("Invalid principal format: {}", e))?;

            // Reject anonymous principal for security
            if principal == Principal::anonymous() {
                return Err("Security Error: Anonymous principal cannot be authorized".to_string());
            }

            // Parse role from string
            let auth_role = match role.to_lowercase().as_str() {
                "owner" => AuthRole::Owner,
                "admin" => AuthRole::Admin,
                "user" => AuthRole::User,
                "readonly" => AuthRole::ReadOnly,
                _ => return Err("Invalid role. Use: owner, admin, user, or readonly".to_string()),
            };

            // Add the user
            Ok(add_user(principal, auth_role))
        }

        /// Remove a user from the authorized users list
        /// Requires Admin role or higher
        #[::ic_cdk_macros::update]
        pub fn remove_authorized_user(principal_text: String) -> Result<String, String> {
            use ::icarus::canister::auth::{remove_user, AuthRole, require_role_or_higher};
            use ::candid::Principal;

            // Require Admin or Owner role
            require_role_or_higher(AuthRole::Admin);

            // Parse the principal
            let principal = Principal::from_text(principal_text)
                .map_err(|e| format!("Invalid principal format: {}", e))?;

            // Remove the user
            Ok(remove_user(principal))
        }

        /// Update a user's role
        /// Requires Admin role or higher
        #[::ic_cdk_macros::update]
        pub fn update_user_role(principal_text: String, new_role: String) -> Result<String, String> {
            use ::icarus::canister::auth::{update_user_role, AuthRole, require_role_or_higher};
            use ::candid::Principal;

            // Require Admin or Owner role
            require_role_or_higher(AuthRole::Admin);

            // Parse the principal
            let principal = Principal::from_text(principal_text)
                .map_err(|e| format!("Invalid principal format: {}", e))?;

            // Security check: reject anonymous principal
            if principal == Principal::anonymous() {
                return Err("Security Error: Anonymous principal cannot have a role".to_string());
            }

            // Parse role from string
            let auth_role = match new_role.to_lowercase().as_str() {
                "owner" => AuthRole::Owner,
                "admin" => AuthRole::Admin,
                "user" => AuthRole::User,
                "readonly" => AuthRole::ReadOnly,
                _ => return Err("Invalid role. Use: owner, admin, user, or readonly".to_string()),
            };

            // Update the role
            Ok(update_user_role(principal, auth_role))
        }

        /// List all authorized users
        /// Requires Admin role or higher
        #[::ic_cdk_macros::query]
        pub fn list_authorized_users() -> String {
            use ::icarus::canister::auth::{get_authorized_users, AuthRole, require_role_or_higher};

            // Require Admin or Owner role
            require_role_or_higher(AuthRole::Admin);

            // Get users and format as JSON
            let users = get_authorized_users();
            ::serde_json::json!({
                "users": users,
                "total": users.len()
            }).to_string()
        }

        /// Get current authentication status
        /// Available to all authenticated users
        #[::ic_cdk_macros::query]
        pub fn get_auth_status() -> String {
            use ::icarus::canister::auth::get_auth_status;

            // Get auth status and serialize to JSON
            ::serde_json::to_string(&get_auth_status())
                .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize auth status: {}"}}"#, e))
        }
    }
}

/// Generate the list_tools function with module configuration
fn generate_list_tools_function(
    tool_entries: &[TokenStream],
    config: &ModuleConfig,
) -> TokenStream {
    let config_title = &config.title;
    let config_website = &config.website_url;

    quote! {
        /// List available MCP tools for discovery
        #[::ic_cdk_macros::query]
        pub fn list_tools() -> String {
            let tools: Vec<::serde_json::Value> = vec![#(#tool_entries),*];

            let mut metadata = ::serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "tools": tools
            });

            // Add optional module configuration fields
            if let Some(title) = #config_title {
                metadata["title"] = ::serde_json::json!(title);
            }

            if let Some(website_url) = #config_website {
                metadata["website_url"] = ::serde_json::json!(website_url);
            }

            metadata.to_string()
        }
    }
}

/// Expand a module marked with #[icarus_module] to automatically generate metadata
pub fn expand_icarus_module(mut input: ItemMod, config: ModuleConfig) -> syn::Result<TokenStream> {
    let _mod_name = &input.ident;
    let _mod_vis = &input.vis;

    // Ensure the module has content
    let content = match &mut input.content {
        Some((_, items)) => items,
        None => {
            // If module has no body, just return it unchanged
            return Ok(quote! { #input });
        }
    };

    // Collect all functions marked with #[icarus_tool]
    let mut tools = Vec::new();
    let mut functions_to_export = Vec::new();

    // Process all functions using optimized helper functions
    for item in content.iter_mut() {
        if let Item::Fn(func) = item {
            // Process all attributes in a single optimized pass
            let attrs = process_function_attributes(func);

            if attrs.has_canister_attribute() {
                // Validate the tool function signature
                let validation = crate::validation::validate_tool_function(
                    func,
                    attrs.has_query,
                    attrs.has_update,
                );
                if !validation.is_valid {
                    let error_msg = validation.errors.join("; ");
                    return Ok(quote! {
                        compile_error!(#error_msg);
                    });
                }

                // Inject authentication if needed
                if attrs.needs_authentication_injection() {
                    inject_authenticate_call(func);
                }

                // Extract function metadata using optimized helpers
                let fn_name = &func.sig.ident;
                let final_description = attrs
                    .description
                    .unwrap_or_else(|| format!("{} function", fn_name));
                let params = extract_function_parameters(func);
                let ret_type = match &func.sig.output {
                    syn::ReturnType::Default => quote! { () },
                    syn::ReturnType::Type(_, ty) => quote! { #ty },
                };

                // Store tool information and export function
                tools.push((
                    fn_name.clone(),
                    final_description,
                    params,
                    ret_type,
                    attrs.has_query,
                    attrs.title,
                    attrs.icon,
                ));
                functions_to_export.push(func.clone());
            }
        }
    }

    // Generate tool metadata entries using optimized builder pattern
    let tool_entries: Vec<_> = tools
        .iter()
        .map(
            |(fn_name, desc, params, _ret_type, _is_query, title, icon)| {
                let builder = ToolMetadataBuilder::new(
                    fn_name.clone(),
                    desc.clone(),
                    params.clone(),
                    title.clone(),
                    icon.clone(),
                );
                builder.generate_metadata()
            },
        )
        .collect();

    // Generate the list_tools function using helper
    let list_tools_fn = generate_list_tools_function(&tool_entries, &config);

    // Generate boilerplate functions using optimized helpers
    let init_fn = generate_init_function();
    let auth_functions = generate_auth_functions();

    // Return the exported functions, metadata function, init function, and auth functions at crate level
    Ok(quote! {
        // Export tool functions at crate level for IC CDK
        #(#functions_to_export)*

        // Export the metadata function
        #list_tools_fn

        // Export the init and post_upgrade functions (always included for security)
        #init_fn

        // Export auth management functions (always included for tool administration)
        #auth_functions
    })
}

/// Optimized type mapping using static lookup tables and `Cow<str>` to reduce allocations
/// Reusable token builder for tool metadata - reduces quote! allocation overhead
struct ToolMetadataBuilder {
    fn_name: syn::Ident,
    description: String,
    params: Vec<(syn::Ident, Box<syn::Type>)>,
    title: Option<String>,
    icon: Option<String>,
}

impl ToolMetadataBuilder {
    fn new(
        fn_name: syn::Ident,
        description: String,
        params: Vec<(syn::Ident, Box<syn::Type>)>,
        title: Option<String>,
        icon: Option<String>,
    ) -> Self {
        Self {
            fn_name,
            description,
            params,
            title,
            icon,
        }
    }

    fn from_refs(
        fn_name: &syn::Ident,
        description: &str,
        params: &[(syn::Ident, Box<syn::Type>)],
        title: &Option<String>,
        icon: &Option<String>,
    ) -> Self {
        Self {
            fn_name: fn_name.clone(),
            description: description.to_string(),
            params: params.to_vec(),
            title: title.clone(),
            icon: icon.clone(),
        }
    }

    /// Generate optimized tool metadata token stream
    fn generate_metadata(&self) -> TokenStream {
        // Pre-allocate vectors with known capacity to reduce allocations
        let mut properties = Vec::with_capacity(self.params.len());
        let mut required = Vec::with_capacity(self.params.len());
        let mut param_order = Vec::with_capacity(self.params.len());
        let mut param_types = Vec::with_capacity(self.params.len());

        // Process parameters in single pass
        for (param_name, param_type) in &self.params {
            let param_name_str = param_name.to_string();
            let type_str = quote!(#param_type).to_string();
            let is_optional = type_str.starts_with("Option <") || type_str.starts_with("Option<");

            let json_type = type_to_json_schema(&type_str);
            let candid_type = type_to_candid_type(&type_str);

            // Build property insertion efficiently
            properties.push(quote! {
                properties.insert(
                    #param_name_str.to_string(),
                    ::serde_json::json!({ "type": #json_type })
                );
            });

            if !is_optional {
                required.push(param_name_str.clone());
            }

            param_order.push(param_name_str);
            param_types.push(candid_type);
        }

        let fn_name = &self.fn_name;
        let desc = &self.description;
        let title = &self.title;
        let icon = &self.icon;

        // Generate required array efficiently
        let required_array = if required.is_empty() {
            quote! { Vec::<&str>::new() }
        } else {
            quote! { vec![#(#required),*] }
        };

        // Generate the complete metadata in one optimized quote! block
        quote! {
            {
                let mut properties = ::serde_json::Map::new();
                #(#properties)*

                let param_style = if #required_array.is_empty() { "empty" } else { "positional" };
                let order_array: Vec<&str> = vec![#(#param_order),*];
                let types_array: Vec<&str> = vec![#(#param_types),*];

                let mut tool_json = ::serde_json::json!({
                    "name": stringify!(#fn_name),
                    "description": #desc,
                    "inputSchema": {
                        "type": "object",
                        "properties": properties,
                        "required": #required_array,
                        "x-icarus-params": {
                            "style": param_style,
                            "order": order_array,
                            "types": types_array
                        }
                    }
                });

                // Add optional fields efficiently
                if let Some(title_value) = #title {
                    tool_json["title"] = ::serde_json::json!(title_value);
                }
                if let Some(icon_value) = #icon {
                    tool_json["icon"] = ::serde_json::json!(icon_value);
                }

                tool_json
            }
        }
    }
}

/// Static lookup table for JSON Schema types - avoids string allocations
const JSON_SCHEMA_MAPPINGS: &[(&str, &str)] = &[
    ("String", "string"),
    ("& str", "string"),
    ("&str", "string"),
    ("str", "string"),
    ("i8", "integer"),
    ("i16", "integer"),
    ("i32", "integer"),
    ("i64", "integer"),
    ("i128", "integer"),
    ("isize", "integer"),
    ("u8", "integer"),
    ("u16", "integer"),
    ("u32", "integer"),
    ("u64", "integer"),
    ("u128", "integer"),
    ("usize", "integer"),
    ("f32", "number"),
    ("f64", "number"),
    ("bool", "boolean"),
    ("Vec <", "array"),
    ("Vec<", "array"),
];

/// Static lookup table for Candid types - avoids string allocations
const CANDID_MAPPINGS: &[(&str, &str)] = &[
    ("String", "text"),
    ("& str", "text"),
    ("&str", "text"),
    ("str", "text"),
    ("i8", "int8"),
    ("i16", "int16"),
    ("i32", "int32"),
    ("i64", "int64"),
    ("i128", "int"),
    ("isize", "int"),
    ("u8", "nat8"),
    ("u16", "nat16"),
    ("u32", "nat32"),
    ("u64", "nat64"),
    ("u128", "nat"),
    ("usize", "nat"),
    ("f32", "float32"),
    ("f64", "float64"),
    ("bool", "bool"),
    ("Principal", "principal"),
    ("Vec", "vec"),
];

/// Optimized type lookup using static table - O(1) average case
fn type_to_json_schema(rust_type: &str) -> &'static str {
    // Fast path: check common exact matches first
    for (pattern, json_type) in JSON_SCHEMA_MAPPINGS {
        if rust_type.contains(pattern) {
            return json_type;
        }
    }
    "string" // Default fallback
}

/// Optimized Candid type lookup using static table - O(1) average case
fn type_to_candid_type(rust_type: &str) -> &'static str {
    // Fast path: check common exact matches first
    for (pattern, candid_type) in CANDID_MAPPINGS {
        if rust_type.contains(pattern) {
            return candid_type;
        }
    }
    "text" // Default fallback
}

/// Expand a crate marked with #[icarus_canister] to automatically generate metadata
pub fn expand_icarus_canister(mut input: File) -> syn::Result<TokenStream> {
    // Collect all functions marked with #[icarus_tool]
    let mut tools = Vec::new();

    // Scan all items in the file
    for item in &input.items {
        if let Item::Fn(func) = item {
            // Check if function has both a canister attribute and icarus_tool
            let has_update = func.attrs.iter().any(|attr| attr.path().is_ident("update"));
            let has_query = func.attrs.iter().any(|attr| attr.path().is_ident("query"));

            if has_update || has_query {
                // Look for icarus_tool attribute first, then fall back to doc comments
                let description = func
                    .attrs
                    .iter()
                    .find_map(extract_tool_description)
                    .or_else(|| {
                        // Fall back to doc comments if no icarus_tool description
                        func.attrs.iter().find_map(|attr| {
                            if attr.path().is_ident("doc") {
                                attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or_else(|| format!("{} function", func.sig.ident));

                // Extract function information
                let fn_name = &func.sig.ident;
                let is_query = has_query;

                // Extract parameters
                let params: Vec<_> = func
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let FnArg::Typed(pat_type) = arg {
                            if let Pat::Ident(ident) = &*pat_type.pat {
                                Some((ident.ident.clone(), pat_type.ty.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                // Extract optional tool metadata
                let title = func.attrs.iter().find_map(extract_tool_title);
                let icon = func.attrs.iter().find_map(extract_tool_icon);

                tools.push((fn_name.clone(), description, params, is_query, title, icon));
            }
        }
    }

    // Generate tool metadata entries using optimized builder pattern
    let tool_entries: Vec<_> = tools
        .iter()
        .map(|(fn_name, desc, params, _is_query, title, icon)| {
            let builder = ToolMetadataBuilder::from_refs(fn_name, desc, params, title, icon);
            builder.generate_metadata()
        })
        .collect();

    // Generate the list_tools function
    let list_tools_fn = quote! {
        /// List available MCP tools for discovery
        #[::ic_cdk_macros::query]
        pub fn list_tools() -> String {
            let tools: Vec<::serde_json::Value> = vec![#(#tool_entries),*];

            ::serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "tools": tools
            }).to_string()
        }
    };

    // Add the list_tools function to the crate items
    let metadata_fn_item: ItemFn = syn::parse2(list_tools_fn.clone()).map_err(|e| {
        syn::Error::new_spanned(
            &list_tools_fn,
            format!("Failed to parse generated list_tools function: {}", e),
        )
    })?;
    input.items.push(Item::Fn(metadata_fn_item));

    // Return the modified crate
    let attrs = &input.attrs;
    let items = &input.items;
    Ok(quote! {
        #(#attrs)*
        #(#items)*
    })
}
