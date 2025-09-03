//! Server macro implementation
//!
//! Generates boilerplate for Icarus servers without MCP protocol handling

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::DeriveInput;

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

        // Server instance for this canister
        thread_local! {
            static SERVER_INSTANCE: std::cell::RefCell<Option<#struct_name>> =
                std::cell::RefCell::new(None);
        }

        impl #struct_name {
            /// Create server configuration
            pub fn config() -> ::icarus::canister::state::ServerConfig {
                ::icarus::canister::state::ServerConfig {
                    name: #name.to_string(),
                    version: #version.to_string(),
                    canister_id: ic_cdk::id(),
                }
            }

            /// Initialize the server instance
            pub fn init_instance() {
                SERVER_INSTANCE.with(|s| {
                    *s.borrow_mut() = Some(#struct_name::new());
                });
            }

            /// Access the server instance
            pub fn with_instance<F, R>(f: F) -> R
            where
                F: FnOnce(&mut #struct_name) -> R
            {
                SERVER_INSTANCE.with(|s| {
                    let mut server = s.borrow_mut();
                    f(server.as_mut().expect("Server not initialized"))
                })
            }
        }

        // Generate canister init function
        #[ic_cdk_macros::init]
        fn __icarus_init() {
            let config = #struct_name::config();
            ::icarus::canister::state::IcarusCanisterState::init(config);

            // Initialize server instance
            #struct_name::init_instance();

            // Initialize tools (will be populated by icarus_tools macro)
            #struct_name::__register_tools();
        }

        // Generate metadata query endpoint
        #[ic_cdk_macros::query]
        fn __icarus_metadata() -> icarus_core::protocol::IcarusMetadata {
            ::icarus::canister::endpoints::icarus_metadata()
        }

        // HTTP gateway endpoint
        #[ic_cdk_macros::query]
        fn http_request(req: ::icarus::canister::endpoints::HttpRequest) -> ::icarus::canister::endpoints::HttpResponse {
            ::icarus::canister::endpoints::http_request(req)
        }

        // Generate pre/post upgrade hooks
        #[ic_cdk_macros::pre_upgrade]
        fn __icarus_pre_upgrade() {
            // State is automatically preserved in stable memory
        }

        #[ic_cdk_macros::post_upgrade]
        fn __icarus_post_upgrade() {
            let config = #struct_name::config();
            ::icarus::canister::state::IcarusCanisterState::init(config);

            // Initialize server instance
            #struct_name::init_instance();

            // Re-register tools
            #struct_name::__register_tools();
        }
    }
}
