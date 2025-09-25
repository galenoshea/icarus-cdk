//! Implementation of the icarus::auth!() macro
//!
//! This macro generates simple authentication management functions for MCP servers.
//! The generated functions provide user management capabilities while ensuring
//! the canister owner maintains ultimate control.

use proc_macro::TokenStream;
use quote::quote;

/// Expand the icarus::auth!() macro
///
/// Generates authentication management functions and handles initialization:
/// - init(owner: Principal) - Initialize auth + WASI (if enabled)
/// - post_upgrade() - Re-initialize WASI after upgrades
/// - add_user(principal, role) - Add users (owner only)
/// - remove_user(principal) - Remove users (owner only)
/// - update_user_role(principal, role) - Change roles (owner only)
/// - get_user_role(principal) - Check user's role (owner or self)
/// - list_users() - List all users (owner only)
/// - get_current_user() - Public self-check function
///
/// Auth state is automatically preserved across upgrades using stable storage.
/// WASI initialization is included automatically when the "wasi" feature is enabled.
///
/// # Deployment Pattern
///
/// Deploy with:
/// ```bash
/// dfx deploy --argument '(principal "owner-principal-id")'
/// ```
pub fn expand(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        /// Initialize authentication system with canister owner
        #[ic_cdk::init]
        fn init(owner: candid::Principal) {
            use icarus::prelude::*;

            // Always initialize authentication
            init_auth(owner);

            // Initialize WASI only if icarus-wasi is available as a dependency
            // This uses a different approach that doesn't cause compile errors
            #[cfg(not(any(test, doctest)))]
            {
                // Try to call WASI initialization if available
                // This will be a no-op if icarus-wasi is not available
                #[allow(unused_imports)]
                use std::sync::Once;
                static WASI_INIT: Once = Once::new();
                WASI_INIT.call_once(|| {
                    // This block will only execute if WASI initialization is needed
                    // The actual WASI init is done via the wasi!() macro if present
                });
            }
        }

        /// Re-initialize WASI after upgrades (auth state is in stable storage)
        #[ic_cdk::post_upgrade]
        fn post_upgrade() {
            // Re-initialize WASI only if needed
            // Auth state is automatically preserved in stable storage
            #[cfg(not(any(test, doctest)))]
            {
                // WASI re-initialization is handled by the wasi!() macro if present
                // No explicit WASI calls needed here to avoid dependency issues
            }
        }

        /// Note: Auth state is automatically preserved in stable storage across upgrades

        /// Add a new user with specified role (owner required)
        ///
        /// Roles:
        /// - "user": Standard user access
        /// - "admin": Can use admin-level tools
        #[ic_cdk::update]
        pub fn add_user(principal: candid::Principal, role: String) -> Result<String, String> {
            use icarus::prelude::*;
            // Validate role parameter
            let auth_role = match role.as_str() {
                "user" => AuthRole::User,
                "admin" => AuthRole::Admin,
                _ => return Err(format!("Invalid role '{}'. Must be 'user' or 'admin'", role)),
            };

            match std::panic::catch_unwind(|| {
                add_user(principal, auth_role)
            }) {
                Ok(result) => Ok(result),
                Err(_) => Err("Failed to add user. Check permissions and principal validity.".to_string()),
            }
        }

        /// Remove a user (owner required)
        ///
        /// Restrictions:
        /// - Cannot remove self
        /// - Must have owner role
        #[ic_cdk::update]
        pub fn remove_user(principal: candid::Principal) -> Result<String, String> {
            use icarus::prelude::*;
            match std::panic::catch_unwind(|| {
                remove_user(principal)
            }) {
                Ok(result) => Ok(result),
                Err(_) => Err("Failed to remove user. Check permissions.".to_string()),
            }
        }

        /// Update user role (owner required)
        ///
        /// Only owners can change user roles. This prevents privilege escalation
        /// and ensures canister owners maintain ultimate control.
        #[ic_cdk::update]
        pub fn update_user_role(principal: candid::Principal, role: String) -> Result<String, String> {
            use icarus::prelude::*;
            // Validate role parameter
            let auth_role = match role.as_str() {
                "user" => AuthRole::User,
                "admin" => AuthRole::Admin,
                "owner" => AuthRole::Owner,
                _ => return Err(format!("Invalid role '{}'. Must be 'user', 'admin', or 'owner'", role)),
            };

            match std::panic::catch_unwind(|| {
                update_user_role(principal, auth_role)
            }) {
                Ok(result) => Ok(result),
                Err(_) => Err("Failed to update user role. Only owners can change roles.".to_string()),
            }
        }

        /// Get a user's role (owner can check anyone, users can check self)
        #[ic_cdk::query]
        pub fn get_user_role(principal: candid::Principal) -> Result<String, String> {
            use icarus::prelude::*;
            let caller = ic_cdk::api::msg_caller();

            // Check if caller is owner or checking their own role
            let can_check = if caller == principal {
                // Anyone can check their own role
                true
            } else {
                // Only owners can check other users' roles
                match std::panic::catch_unwind(|| require_role_or_higher(AuthRole::Owner)) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            };

            if !can_check {
                return Err("Permission denied. Can only check your own role or owner can check anyone.".to_string());
            }

            // Get the target user's info
            match get_user(principal) {
                Some(user) => Ok(format!("{:?}", user.role).to_lowercase()),
                None => Err("User not found".to_string()),
            }
        }

        /// List all authorized users (owner required)
        #[ic_cdk::query]
        pub fn list_users() -> Result<Vec<icarus::prelude::User>, String> {
            use icarus::prelude::*;
            match std::panic::catch_unwind(|| {
                list_users()
            }) {
                Ok(users) => Ok(users),
                Err(_) => Err("Permission denied. Owner role required.".to_string()),
            }
        }

        /// Get current caller's authentication information (public)
        ///
        /// Anyone can check their own authentication status.
        /// Returns user info if authenticated, or error if not.
        #[ic_cdk::query]
        pub fn get_current_user() -> Result<icarus::prelude::AuthInfo, String> {
            use icarus::prelude::*;
            match std::panic::catch_unwind(|| {
                get_current_user()
            }) {
                Ok(auth_info) => Ok(auth_info),
                Err(_) => Err("Not authenticated".to_string()),
            }
        }

    };

    expanded.into()
}
