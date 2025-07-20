//! Tools macro implementation
//! 
//! Generates Candid methods from tool definitions

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ImplItem, FnArg, Pat, Type, ItemMod, Item, File, ItemFn};

pub fn expand_icarus_tools(_attr: TokenStream, input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    
    // Collect all methods marked with icarus_tool
    let mut tool_registrations = vec![];
    let mut candid_methods = vec![];
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Check if method has icarus_tool attribute
            for attr in &method.attrs {
                if attr.path().is_ident("icarus_tool") {
                    let method_name = &method.sig.ident;
                    let tool_name = extract_tool_name(attr).unwrap_or_else(|| method_name.to_string());
                    let is_query = extract_is_query(attr);
                    let is_public = extract_is_public(attr);
                    let is_async = method.sig.asyncness.is_some();
                    
                    // Determine if this is a query based on receiver or attribute
                    let is_query = is_query.unwrap_or_else(|| {
                        method.sig.receiver().map(|r| {
                            // If it takes &self, it's a query. If &mut self, it's an update
                            match r {
                                syn::Receiver { mutability: None, .. } => true,
                                syn::Receiver { mutability: Some(_), .. } => false,
                            }
                        }).unwrap_or(true)
                    });
                    
                    // Extract parameters
                    let params: Vec<_> = method.sig.inputs.iter()
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
                    
                    // Register tool in canister state
                    tool_registrations.push(quote! {
                        icarus_canister::state::STATE.with(|s| {
                            if let Some(state) = s.borrow_mut().as_mut() {
                                state.tools.insert(
                                    #tool_name.to_string(),
                                    icarus_canister::state::ToolState {
                                        name: #tool_name.to_string(),
                                        enabled: true,
                                        call_count: 0,
                                        is_query: #is_query,
                                    }
                                );
                            }
                        });
                    });
                    
                    // Generate Candid method based on query/update and async/sync
                    let candid_method = match (is_query, is_async) {
                        (true, true) => generate_async_query_method(&tool_name, method_name, &params, is_public),
                        (true, false) => generate_query_method(&tool_name, method_name, &params, is_public),
                        (false, true) => generate_async_update_method(&tool_name, method_name, &params, is_public),
                        (false, false) => generate_update_method(&tool_name, method_name, &params, is_public),
                    };
                    
                    candid_methods.push(candid_method);
                }
            }
        }
    }
    
    // Generate the implementation with tool registration
    quote! {
        #input
        
        impl #self_ty {
            /// Register all tools with the canister state
            pub fn __register_tools() {
                #(#tool_registrations)*
            }
        }
        
        // Generate Candid methods
        #(#candid_methods)*
    }
}

fn generate_query_method(_tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)], is_public: bool) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    let access_check = if is_public {
        quote! {}
    } else {
        quote! {
            // Verify caller is the canister owner
            icarus_canister::assert_owner();
        }
    };

    quote! {
        // Query methods must be synchronous in ICP
        #[ic_cdk::query]
        fn #method_name(#(#param_list),*) -> String {
            #access_check
            
            SERVER_INSTANCE.with(|s| {
                let server = s.borrow();
                let server = server.as_ref().expect("Server not initialized");
                
                // Call the sync method directly
                match server.#method_name(#(#param_pass),*) {
                    Ok(value) => value.to_string(),
                    Err(e) => serde_json::json!({
                        "error": e.to_string()
                    }).to_string()
                }
            })
        }
    }
}

fn generate_update_method(_tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)], is_public: bool) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    let access_check = if is_public {
        quote! {}
    } else {
        quote! {
            // Verify caller is the canister owner
            icarus_canister::assert_owner();
        }
    };

    quote! {
        // Sync update method
        #[ic_cdk::update]
        fn #method_name(#(#param_list),*) -> String {
            #access_check
            
            SERVER_INSTANCE.with(|s| {
                let mut server = s.borrow_mut();
                let server = server.as_mut().expect("Server not initialized");
                
                // Call the sync method directly
                match server.#method_name(#(#param_pass),*) {
                    Ok(value) => value.to_string(),
                    Err(e) => serde_json::json!({
                        "error": e.to_string()
                    }).to_string()
                }
            })
        }
    }
}

fn extract_tool_name(attr: &syn::Attribute) -> Option<String> {
    // Parse the attribute to extract name = "tool_name"
    let mut name = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            if let Ok(value) = meta.value() {
                if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                    name = Some(lit_str.value());
                }
            }
        }
        Ok(())
    });
    
    name
}

fn extract_is_query(attr: &syn::Attribute) -> Option<bool> {
    // Parse the attribute to extract is_query = true/false
    let mut is_query = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("is_query") {
            if let Ok(value) = meta.value() {
                if let Ok(lit_bool) = value.parse::<syn::LitBool>() {
                    is_query = Some(lit_bool.value());
                }
            }
        }
        Ok(())
    });
    
    is_query
}

fn extract_is_public(attr: &syn::Attribute) -> bool {
    // Parse the attribute to extract public = true/false, default false
    let mut is_public = false;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("public") {
            if let Ok(value) = meta.value() {
                if let Ok(lit_bool) = value.parse::<syn::LitBool>() {
                    is_public = lit_bool.value();
                }
            } else {
                // If just "public" without value, assume true
                is_public = true;
            }
        }
        Ok(())
    });
    
    is_public
}

// Generate async query method - This will error at compile time with helpful message
fn generate_async_query_method(_tool_name: &str, method_name: &syn::Ident, _params: &[(syn::Ident, Box<Type>)], _is_public: bool) -> TokenStream {
    quote! {
        // This will generate a compile error with a helpful message
        compile_error!(concat!(
            "Query method '", stringify!(#method_name), "' cannot be async. ",
            "IC query methods must be synchronous. ",
            "Either remove 'async' from the method or change it to an update method by removing 'is_query = true'."
        ));
    }
}

// Generate async update method
fn generate_async_update_method(tool_name: &str, method_name: &syn::Ident, params: &[(syn::Ident, Box<Type>)], is_public: bool) -> TokenStream {
    // Generate parameter list for Candid method
    let param_list = params.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    // Generate parameter passing to actual method
    let param_pass = params.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    let access_check = if is_public {
        quote! {}
    } else {
        quote! {
            // Verify caller is the canister owner
            icarus_canister::assert_owner();
        }
    };

    quote! {
        // Async update method - IC supports async updates
        #[ic_cdk::update]
        async fn #method_name(#(#param_list),*) -> String {
            #access_check
            
            // We need to extract the future before awaiting to avoid RefCell issues
            let fut = SERVER_INSTANCE.with(|s| {
                let mut server = s.borrow_mut();
                let server = server.as_mut().expect("Server not initialized");
                
                // For now, return a placeholder until we find a better solution
                // The issue is that we can't hold a mutable borrow across an await point
                Box::pin(std::future::ready(Ok(serde_json::json!({
                    "status": "success",
                    "message": "Async update called",
                    "tool": #tool_name
                }))))
            });
            
            let result = fut.await;
            
            match result {
                Ok(value) => value.to_string(),
                Err(e) => serde_json::json!({
                    "error": e.to_string()
                }).to_string()
            }
        }
    }
}

/// Expand a module marked with #[icarus_module] to automatically generate metadata
pub fn expand_icarus_module(mut input: ItemMod) -> TokenStream {
    let mod_name = &input.ident;
    let mod_vis = &input.vis;
    
    // Ensure the module has content
    let content = match &mut input.content {
        Some((_, items)) => items,
        None => {
            // If module has no body, just return it unchanged
            return quote! { #input };
        }
    };
    
    // Collect all functions marked with #[icarus_tool]
    let mut tools = Vec::new();
    let mut functions_to_export = Vec::new();
    
    for item in content.iter() {
        if let Item::Fn(func) = item {
            // Check if function has both a canister attribute and icarus_tool
            let has_update = func.attrs.iter().any(|attr| attr.path().is_ident("update"));
            let has_query = func.attrs.iter().any(|attr| attr.path().is_ident("query"));
            
            if has_update || has_query {
                // Clone the function to export at crate level
                let mut exported_func = func.clone();
                
                // Look for the doc comment that icarus_tool generates
                let description = func.attrs.iter()
                    .find_map(|attr| {
                        if attr.path().is_ident("doc") {
                            attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| format!("{} function", func.sig.ident));
                
                // Extract function information
                let fn_name = &func.sig.ident;
                let is_query = has_query;
                
                // Extract parameters
                let params: Vec<_> = func.sig.inputs.iter()
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
                
                // Extract return type
                let ret_type = match &func.sig.output {
                    syn::ReturnType::Default => quote! { () },
                    syn::ReturnType::Type(_, ty) => quote! { #ty },
                };
                
                tools.push((fn_name.clone(), description, params, ret_type, is_query));
                functions_to_export.push(exported_func);
            }
        }
    }
    
    // Generate tool metadata entries
    let tool_entries: Vec<_> = tools.iter().map(|(fn_name, desc, params, _ret_type, _is_query)| {
        let mut properties = quote! {};
        let mut required = Vec::new();
        
        for (param_name, param_type) in params {
            let param_name_str = param_name.to_string();
            let is_optional = quote!(#param_type).to_string().starts_with("Option <");
            let json_type = type_to_json_schema(&quote!(#param_type).to_string());
            
            properties = quote! {
                #properties
                properties.insert(
                    #param_name_str.to_string(),
                    ::serde_json::json!({ "type": #json_type })
                );
            };
            
            if !is_optional {
                required.push(param_name_str);
            }
        }
        
        let required_array = if required.is_empty() {
            quote! { Vec::<&str>::new() }
        } else {
            quote! { vec![#(#required),*] }
        };
        
        quote! {
            {
                let mut properties = ::serde_json::Map::new();
                #properties
                
                ::serde_json::json!({
                    "name": stringify!(#fn_name),
                    "description": #desc,
                    "inputSchema": {
                        "type": "object",
                        "properties": properties,
                        "required": #required_array
                    }
                })
            }
        }
    }).collect();
    
    // Generate the get_metadata function
    let get_metadata_fn = quote! {
        /// Get canister metadata for tool discovery
        #[::ic_cdk_macros::query]
        pub fn get_metadata() -> String {
            let tools = vec![#(#tool_entries),*];
            
            ::serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "tools": tools
            }).to_string()
        }
    };
    
    // Return both the exported functions and the get_metadata function at crate level
    quote! {
        // Export tool functions at crate level for IC CDK
        #(#functions_to_export)*
        
        // Export the metadata function
        #get_metadata_fn
    }
}

/// Convert Rust type string to JSON Schema type
fn type_to_json_schema(rust_type: &str) -> &'static str {
    match rust_type {
        s if s.contains("String") || s.contains("& str") => "string",
        s if s.contains("i32") || s.contains("i64") || s.contains("u32") || s.contains("u64") || s.contains("usize") => "integer",
        s if s.contains("f32") || s.contains("f64") => "number",
        s if s.contains("bool") => "boolean",
        s if s.contains("Vec <") => "array",
        _ => "string", // Default to string for unknown types
    }
}

/// Expand a crate marked with #[icarus_canister] to automatically generate metadata
pub fn expand_icarus_canister(mut input: File) -> TokenStream {
    // Collect all functions marked with #[icarus_tool]
    let mut tools = Vec::new();
    
    // Scan all items in the file
    for item in &input.items {
        if let Item::Fn(func) = item {
            // Check if function has both a canister attribute and icarus_tool
            let has_update = func.attrs.iter().any(|attr| attr.path().is_ident("update"));
            let has_query = func.attrs.iter().any(|attr| attr.path().is_ident("query"));
            
            if has_update || has_query {
                // Look for the icarus_tool doc comment
                let description = func.attrs.iter()
                    .find_map(|attr| {
                        if attr.path().is_ident("doc") {
                            attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| format!("{} function", func.sig.ident));
                
                // Extract function information
                let fn_name = &func.sig.ident;
                let is_query = has_query;
                
                // Extract parameters
                let params: Vec<_> = func.sig.inputs.iter()
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
                
                tools.push((fn_name.clone(), description, params, is_query));
            }
        }
    }
    
    // Generate tool metadata entries
    let tool_entries: Vec<_> = tools.iter().map(|(fn_name, desc, params, _is_query)| {
        let mut properties = quote! {};
        let mut required = Vec::new();
        
        for (param_name, param_type) in params {
            let param_name_str = param_name.to_string();
            let is_optional = quote!(#param_type).to_string().starts_with("Option <");
            let json_type = type_to_json_schema(&quote!(#param_type).to_string());
            
            properties = quote! {
                #properties
                properties.insert(
                    #param_name_str.to_string(),
                    ::serde_json::json!({ "type": #json_type })
                );
            };
            
            if !is_optional {
                required.push(param_name_str);
            }
        }
        
        let required_array = if required.is_empty() {
            quote! { Vec::<&str>::new() }
        } else {
            quote! { vec![#(#required),*] }
        };
        
        quote! {
            {
                let mut properties = ::serde_json::Map::new();
                #properties
                
                ::serde_json::json!({
                    "name": stringify!(#fn_name),
                    "description": #desc,
                    "inputSchema": {
                        "type": "object",
                        "properties": properties,
                        "required": #required_array
                    }
                })
            }
        }
    }).collect();
    
    // Generate the get_metadata function
    let get_metadata_fn = quote! {
        /// Get canister metadata for tool discovery
        #[::ic_cdk_macros::query]
        pub fn get_metadata() -> String {
            let tools = vec![#(#tool_entries),*];
            
            ::serde_json::json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "tools": tools
            }).to_string()
        }
    };
    
    // Add the get_metadata function to the crate items
    let metadata_fn_item: ItemFn = syn::parse2(get_metadata_fn.clone()).unwrap();
    input.items.push(Item::Fn(metadata_fn_item));
    
    // Return the modified crate
    let attrs = &input.attrs;
    let items = &input.items;
    quote! {
        #(#attrs)*
        #(#items)*
    }
}