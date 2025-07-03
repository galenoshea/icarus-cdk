//! Tools macro implementation

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, ImplItem};

pub fn expand_icarus_tools(_attr: TokenStream, input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    
    // Collect all methods marked with icarus_tool
    let mut tool_registrations = vec![];
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Check if method has icarus_tool attribute
            for attr in &method.attrs {
                if attr.path().is_ident("icarus_tool") {
                    let method_name = &method.sig.ident;
                    let tool_name = extract_tool_name(attr).unwrap_or_else(|| method_name.to_string());
                    
                    tool_registrations.push(quote! {
                        {
                            let registration = icarus_canister::tools::ToolRegistration {
                                name: #tool_name.to_string(),
                                description: format!("{} tool", #tool_name),
                                function: Box::new(move |args: serde_json::Value| {
                                    Box::pin(async move {
                                        // For now, just return an error - the actual implementation
                                        // would need to parse args based on method signature
                                        Err(icarus_core::error::ToolError::internal(
                                            "Tool dispatch not yet implemented"
                                        ))
                                    })
                                }),
                            };
                            registry.register(registration);
                        }
                    });
                }
            }
        }
    }
    
    // Generate the implementation with tool registration
    quote! {
        #input
        
        impl #self_ty {
            fn __register_tools() {
                <#self_ty>::with_registry_mut(|registry| {
                    #(#tool_registrations)*
                });
            }
        }
    }
}

fn extract_tool_name(attr: &syn::Attribute) -> Option<String> {
    let mut name = None;
    
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Ok(()) // Ignore other attributes
        }
    });
    
    name
}