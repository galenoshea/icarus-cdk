//! Authentication and authorization system for Icarus canisters
//!
//! Provides a simple authentication system with role-based access control
//! and secure principal management for MCP servers.

#[cfg(feature = "canister")]
use crate::{memory_id, stable_storage};
#[cfg(feature = "canister")]
use candid::{CandidType, Deserialize, Principal};
#[cfg(feature = "canister")]
use ic_stable_structures::memory_manager::VirtualMemory;
#[cfg(feature = "canister")]
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
#[cfg(feature = "canister")]
use serde::Serialize;

#[cfg(feature = "canister")]
type Memory = VirtualMemory<DefaultMemoryImpl>;

/// User entry for simple role-based access control
#[cfg(feature = "canister")]
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct User {
    pub principal: Principal,
    pub added_at: u64,
    pub added_by: Principal,
    pub role: AuthRole,
    pub active: bool,
}

/// Role-based access control for MCP servers
#[cfg(feature = "canister")]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, CandidType)]
pub enum AuthRole {
    Owner, // Full access, can manage all users
    Admin, // Can use admin-level tools
    User,  // Can use regular tools
}

/// Authentication result
#[cfg(feature = "canister")]
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AuthInfo {
    pub principal: String,
    pub role: AuthRole,
    pub is_authenticated: bool,
}

// Implement Storable for User
#[cfg(feature = "canister")]
impl ic_stable_structures::Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    fn into_bytes(self) -> std::vec::Vec<u8> {
        candid::encode_one(self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

// Implement IcarusStorable for User
#[cfg(feature = "canister")]
impl crate::storage::IcarusStorable for User {}

// Declare stable storage for authentication system
#[cfg(feature = "canister")]
stable_storage! {
    AUTH_USERS: StableBTreeMap<Principal, User, Memory> = memory_id!(10);
}

/// Initialize the authentication system with the canister owner
#[cfg(feature = "canister")]
pub fn init_auth(owner: Principal) {
    // Security check: prevent anonymous principal from being owner
    if owner == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot be set as owner");
    }

    AUTH_USERS.with(|users| {
        let owner_entry = User {
            principal: owner,
            added_at: ic_cdk::api::time(),
            added_by: owner, // Self-added
            role: AuthRole::Owner,
            active: true,
        };
        users.borrow_mut().insert(owner, owner_entry);
    });
}

/// Validate authentication and return auth info, or trap on failure
#[cfg(feature = "canister")]
pub fn authenticate() -> AuthInfo {
    let caller = ic_cdk::api::msg_caller();

    // Security check: anonymous principal is never authenticated
    if caller == Principal::anonymous() {
        ic_cdk::trap("Access denied: Anonymous principal cannot be authenticated");
    }

    AUTH_USERS.with(|users| {
        let auth_entry = if let Some(entry) = users.borrow().get(&caller) {
            entry
        } else {
            ic_cdk::trap(format!(
                "Access denied: Principal {} not authorized",
                caller.to_text()
            ));
        };

        if !auth_entry.active {
            ic_cdk::trap("Access denied: account deactivated");
        }

        AuthInfo {
            principal: caller.to_text(),
            role: auth_entry.role,
            is_authenticated: true,
        }
    })
}

/// Check if caller has specific role or higher (hierarchical)
/// Owner > Admin > User
#[cfg(feature = "canister")]
pub fn require_role_or_higher(minimum_role: AuthRole) -> AuthInfo {
    let auth_info = authenticate();

    let has_permission = matches!(
        (&auth_info.role, &minimum_role),
        (AuthRole::Owner, _)
            | (AuthRole::Admin, AuthRole::Admin | AuthRole::User)
            | (AuthRole::User, AuthRole::User)
    );

    if has_permission {
        auth_info
    } else {
        ic_cdk::trap(format!(
            "Insufficient permissions: {:?} or higher required",
            minimum_role
        ));
    }
}

/// Add a new user (requires Owner role)
#[cfg(feature = "canister")]
pub fn add_user(principal: Principal, role: AuthRole) -> String {
    // Security check: prevent anonymous principal from being added
    if principal == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot be authorized");
    }

    require_role_or_higher(AuthRole::Owner);
    let caller = ic_cdk::api::msg_caller();

    AUTH_USERS.with(|users| {
        if users.borrow().contains_key(&principal) {
            ic_cdk::trap("Principal already authorized");
        }

        let auth_entry = User {
            principal,
            added_at: ic_cdk::api::time(),
            added_by: caller,
            role,
            active: true,
        };

        users.borrow_mut().insert(principal, auth_entry);

        format!(
            "Principal {} added with role {:?}",
            principal.to_text(),
            role
        )
    })
}

/// Remove a user (requires Owner role)
#[cfg(feature = "canister")]
pub fn remove_user(principal: Principal) -> String {
    require_role_or_higher(AuthRole::Owner);
    let caller = ic_cdk::api::msg_caller();

    // Prevent self-removal
    if principal == caller {
        ic_cdk::trap("Cannot remove yourself");
    }

    AUTH_USERS.with(|users| {
        if users.borrow().contains_key(&principal) {
            users.borrow_mut().remove(&principal);
            format!("Principal {} removed", principal.to_text())
        } else {
            ic_cdk::trap(format!("User {} not found", principal.to_text()))
        }
    })
}

/// Update user role (requires Owner role)
#[cfg(feature = "canister")]
pub fn update_user_role(principal: Principal, new_role: AuthRole) -> String {
    // Security check: prevent anonymous principal from having any role
    if principal == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot have a role");
    }

    require_role_or_higher(AuthRole::Owner);

    AUTH_USERS.with(|users| {
        if let Some(mut auth_entry) = users.borrow().get(&principal) {
            let old_role = auth_entry.role;
            auth_entry.role = new_role;
            users.borrow_mut().insert(principal, auth_entry);

            format!(
                "Principal {} role updated from {:?} to {:?}",
                principal.to_text(),
                old_role,
                new_role
            )
        } else {
            ic_cdk::trap("Principal not found");
        }
    })
}

/// Get all authorized users (requires Owner role)
#[cfg(feature = "canister")]
pub fn list_users() -> Vec<User> {
    require_role_or_higher(AuthRole::Owner);

    AUTH_USERS.with(|users| {
        users
            .borrow()
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    })
}

/// Get current caller's user info
#[cfg(feature = "canister")]
pub fn get_current_user() -> AuthInfo {
    authenticate()
}

/// Get specific user by principal
#[cfg(feature = "canister")]
pub fn get_user(principal: Principal) -> Option<User> {
    AUTH_USERS.with(|users| users.borrow().get(&principal))
}

/// Check if a principal is authenticated (required by derive macros)
#[cfg(feature = "canister")]
pub fn is_authenticated(principal: &Principal) -> bool {
    if *principal == Principal::anonymous() {
        return false;
    }

    AUTH_USERS.with(|users| {
        users
            .borrow()
            .get(principal)
            .is_some_and(|user| user.active)
    })
}

/// Check if a principal is the owner (required by derive macros)
#[cfg(feature = "canister")]
pub fn is_owner(principal: &Principal) -> bool {
    if *principal == Principal::anonymous() {
        return false;
    }

    AUTH_USERS.with(|users| {
        users
            .borrow()
            .get(principal)
            .is_some_and(|user| user.active && matches!(user.role, AuthRole::Owner))
    })
}

// Convenience macros for common auth checks
#[cfg(feature = "canister")]
#[macro_export]
macro_rules! require_auth {
    () => {
        $crate::auth::authenticate();
    };
}

#[cfg(feature = "canister")]
#[macro_export]
macro_rules! require_owner {
    () => {
        $crate::auth::require_role_or_higher($crate::auth::AuthRole::Owner);
    };
}

#[cfg(all(test, feature = "canister"))]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Mock stable storage for testing
    thread_local! {
        static MOCK_USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
        static MOCK_TIME: RefCell<u64> = const { RefCell::new(1000000000000) };
        static MOCK_CALLER: RefCell<Principal> = const { RefCell::new(Principal::anonymous()) };
    }

    fn create_test_principal(text: &str) -> Principal {
        Principal::from_text(text).unwrap_or_else(|_| {
            // For test strings that aren't valid principal text, create a deterministic principal
            let bytes = text.as_bytes();
            let mut padded = [0u8; 29]; // Principal can be up to 29 bytes
            let len = bytes.len().min(29);
            padded[..len].copy_from_slice(&bytes[..len]);
            Principal::from_slice(&padded[..len])
        })
    }

    #[test]
    fn test_user_creation() {
        let owner = create_test_principal("owner");
        let user = User {
            principal: owner,
            added_at: 1000000000000,
            added_by: owner,
            role: AuthRole::Owner,
            active: true,
        };

        assert_eq!(user.principal, owner);
        assert_eq!(user.role, AuthRole::Owner);
        assert!(user.active);
    }

    #[test]
    fn test_auth_role_hierarchy() {
        // Test role equality
        assert_eq!(AuthRole::Owner, AuthRole::Owner);
        assert_eq!(AuthRole::Admin, AuthRole::Admin);
        assert_eq!(AuthRole::User, AuthRole::User);

        // Test role inequality
        assert_ne!(AuthRole::Owner, AuthRole::Admin);
        assert_ne!(AuthRole::Admin, AuthRole::User);
        assert_ne!(AuthRole::Owner, AuthRole::User);
    }

    // Additional tests for completeness...
    #[test]
    fn test_auth_info_creation() {
        let principal = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let auth_info = AuthInfo {
            principal: principal.to_string(),
            role: AuthRole::User,
            is_authenticated: true,
        };

        assert_eq!(auth_info.principal, principal);
        assert_eq!(auth_info.role, AuthRole::User);
        assert!(auth_info.is_authenticated);
    }
}
