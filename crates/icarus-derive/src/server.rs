//! Server macro implementation

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse::Parser;

pub fn expand_icarus_server(args: TokenStream, input: DeriveInput) -> TokenStream {
    // Parse server metadata
    let mut server_name = None;
    let mut server_version = None;
    
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            server_name = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("version") {
            server_version = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Err(meta.error("unsupported icarus_server attribute"))
        }
    });
    
    let _ = parser.parse2(args);
    
    let struct_name = &input.ident;
    let name = server_name.unwrap_or_else(|| "icarus-server".to_string());
    let version = server_version.unwrap_or_else(|| "0.1.0".to_string());
    
    // Generate the expanded code
    quote! {
        #input
        
        // Tool registry for this server
        thread_local! {
            static TOOL_REGISTRY: std::cell::RefCell<icarus_canister::tools::ToolRegistry> = 
                std::cell::RefCell::new(icarus_canister::tools::ToolRegistry::new());
        }
        
        impl #struct_name {
            /// Create server configuration
            pub fn config() -> icarus_canister::state::ServerConfig {
                icarus_canister::state::ServerConfig {
                    name: #name.to_string(),
                    version: #version.to_string(),
                    canister_id: ic_cdk::id(),
                }
            }
            
            /// Get the tool registry
            pub fn with_registry<F, R>(f: F) -> R
            where
                F: FnOnce(&icarus_canister::tools::ToolRegistry) -> R
            {
                TOOL_REGISTRY.with(|r| f(&r.borrow()))
            }
            
            /// Get the tool registry mutably
            pub fn with_registry_mut<F, R>(f: F) -> R
            where
                F: FnOnce(&mut icarus_canister::tools::ToolRegistry) -> R
            {
                TOOL_REGISTRY.with(|r| f(&mut r.borrow_mut()))
            }
        }
        
        // Generate canister init function
        #[ic_cdk_macros::init]
        fn __icarus_init() {
            let config = #struct_name::config();
            icarus_canister::state::IcarusCanisterState::init(config);
            
            // Initialize tools (will be populated by icarus_tools macro)
            #struct_name::__register_tools();
        }
        
        
        // Generate canister query/update methods
        #[ic_cdk_macros::update]
        async fn icarus_mcp_request(request: icarus_core::protocol::IcarusMcpRequest) -> icarus_core::protocol::IcarusMcpResponse {
            // For now, just use the default handler
            icarus_canister::endpoints::icarus_mcp_request(request).await
        }
        
        #[ic_cdk_macros::query]
        fn icarus_capabilities() -> icarus_core::protocol::IcarusServerCapabilities {
            icarus_canister::endpoints::icarus_capabilities()
        }
        
        // Generate pre/post upgrade hooks
        #[ic_cdk_macros::pre_upgrade]
        fn __icarus_pre_upgrade() {
            // State is automatically preserved in stable memory
        }
        
        #[ic_cdk_macros::post_upgrade]
        fn __icarus_post_upgrade() {
            __icarus_init();
        }
    }
}