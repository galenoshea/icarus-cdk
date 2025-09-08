//! Authentication and authorization system for Icarus canisters
//!
//! Provides a comprehensive authentication system with audit trails,
//! role-based access control, and secure principal management.

use crate::{memory_id, stable_storage, IcarusStorable};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use serde::Serialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

/// User entry with full audit trail
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
#[icarus_storable(unbounded)]
pub struct User {
    pub principal: Principal,
    pub added_at: u64,
    pub added_by: Principal,
    pub role: AuthRole,
    pub active: bool,
    pub last_access: Option<u64>,
    pub access_count: u64,
}

/// Role-based access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub enum AuthRole {
    Owner,    // Full access, can manage all users
    Admin,    // Can add/remove users, view audit logs
    User,     // Normal tool access
    ReadOnly, // Query-only access
}

/// Authentication result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AuthInfo {
    pub principal: String,
    pub role: AuthRole,
    pub is_authenticated: bool,
    pub last_access: Option<u64>,
    pub access_count: u64,
    pub message: String,
}

/// Audit log entry for authentication events
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
#[icarus_storable(unbounded)]
pub struct AuthAuditEntry {
    pub id: String,
    pub timestamp: u64,
    pub action: AuthAction,
    pub principal: Principal,
    pub target_principal: Option<Principal>,
    pub performed_by: Principal,
    pub success: bool,
    pub details: String,
}

/// Authentication actions for audit logging
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum AuthAction {
    AddUser,
    RemoveUser,
    UpdateRole,
    DeactivateUser,
    ReactivateUser,
    AccessGranted,
    AccessDenied,
    ViewAuditLog,
}

// Declare stable storage for authentication system
stable_storage! {
    AUTH_USERS: StableBTreeMap<Principal, User, Memory> = memory_id!(10);
    AUTH_AUDIT: StableBTreeMap<String, AuthAuditEntry, Memory> = memory_id!(11);
    AUTH_COUNTER: u64 = 0;
}

/// Initialize the authentication system with the canister owner
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
            last_access: None,
            access_count: 0,
        };
        users.borrow_mut().insert(owner, owner_entry);
    });

    log_auth_action(
        AuthAction::AddUser,
        owner,
        Some(owner),
        owner,
        true,
        "Initial owner setup".to_string(),
    );
}

/// Validate authentication and return detailed auth info, or trap on failure
pub fn authenticate() -> AuthInfo {
    let caller = ic_cdk::caller();

    AUTH_USERS.with(|users| {
        // Get and clone the entry to avoid borrow conflicts
        let mut auth_entry = if let Some(entry) = users.borrow().get(&caller) {
            entry.clone()
        } else {
            log_auth_action(
                AuthAction::AccessDenied,
                caller,
                None,
                caller,
                false,
                "Principal not found in authorized users".to_string(),
            );

            ic_cdk::trap(&format!(
                "Access denied: Principal {} not authorized",
                caller.to_text()
            ));
        };

        if !auth_entry.active {
            log_auth_action(
                AuthAction::AccessDenied,
                caller,
                None,
                caller,
                false,
                "User account deactivated".to_string(),
            );

            ic_cdk::trap("Access denied: account deactivated");
        }

        // Update access tracking
        auth_entry.last_access = Some(ic_cdk::api::time());
        auth_entry.access_count += 1;
        users.borrow_mut().insert(caller, auth_entry.clone());

        log_auth_action(
            AuthAction::AccessGranted,
            caller,
            None,
            caller,
            true,
            format!("Access granted with role: {:?}", auth_entry.role),
        );

        AuthInfo {
            principal: caller.to_text(),
            role: auth_entry.role,
            is_authenticated: true,
            last_access: auth_entry.last_access,
            access_count: auth_entry.access_count,
            message: "Access granted".to_string(),
        }
    })
}

/// Check if caller has specific role or higher (hierarchical)
/// Owner > Admin > User > ReadOnly
pub fn require_role_or_higher(minimum_role: AuthRole) -> AuthInfo {
    let auth_info = authenticate();

    let has_permission = matches!(
        (&auth_info.role, &minimum_role),
        (AuthRole::Owner, _)
            | (
                AuthRole::Admin,
                AuthRole::Admin | AuthRole::User | AuthRole::ReadOnly
            )
            | (AuthRole::User, AuthRole::User | AuthRole::ReadOnly)
            | (AuthRole::ReadOnly, AuthRole::ReadOnly)
    );

    if has_permission {
        auth_info
    } else {
        ic_cdk::trap(&format!(
            "Insufficient permissions: {:?} or higher required",
            minimum_role
        ));
    }
}

/// Check if caller has exactly the specified role
pub fn require_exact_role(role: AuthRole) -> AuthInfo {
    let auth_info = authenticate();
    if matches!(auth_info.role, ref r if *r == role) {
        auth_info
    } else {
        ic_cdk::trap(&format!(
            "Requires exactly {:?} role, but caller has {:?}",
            role, auth_info.role
        ));
    }
}

/// Check if caller has any of the specified roles
pub fn require_any_of_roles(roles: &[AuthRole]) -> AuthInfo {
    let auth_info = authenticate();
    if roles.contains(&auth_info.role) {
        auth_info
    } else {
        ic_cdk::trap(&format!(
            "Requires one of: {:?}, but caller has {:?}",
            roles, auth_info.role
        ));
    }
}

/// Check if caller does NOT have any of the excluded roles
pub fn require_none_of_roles(excluded: &[AuthRole]) -> AuthInfo {
    let auth_info = authenticate();
    if !excluded.contains(&auth_info.role) {
        auth_info
    } else {
        ic_cdk::trap(&format!("Role {:?} is not allowed here", auth_info.role));
    }
}

/// Add a new user (requires Admin or Owner role)
pub fn add_user(principal: Principal, role: AuthRole) -> String {
    // Security check: prevent anonymous principal from being added
    if principal == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot be authorized");
    }

    let auth_info = require_role_or_higher(AuthRole::Admin);
    let caller = ic_cdk::caller();

    // Prevent self-elevation (Admins can't create Owners)
    if matches!(role, AuthRole::Owner) && !matches!(auth_info.role, AuthRole::Owner) {
        ic_cdk::trap("Only owners can create other owners");
    }

    AUTH_USERS.with(|users| {
        if users.borrow().contains_key(&principal) {
            ic_cdk::trap("Principal already authorized");
        }

        let auth_entry = User {
            principal,
            added_at: ic_cdk::api::time(),
            added_by: caller,
            role: role.clone(),
            active: true,
            last_access: None,
            access_count: 0,
        };

        users.borrow_mut().insert(principal, auth_entry);

        log_auth_action(
            AuthAction::AddUser,
            principal,
            Some(principal),
            caller,
            true,
            format!("User added with role: {:?}", role),
        );

        format!(
            "Principal {} added with role {:?} by {}",
            principal.to_text(),
            role,
            caller.to_text()
        )
    })
}

/// Remove a user (requires Admin or Owner role)
pub fn remove_user(principal: Principal) -> String {
    let auth_info = require_role_or_higher(AuthRole::Admin);
    let caller = ic_cdk::caller();

    AUTH_USERS.with(|users| {
        // First, check the user and validate permissions in a separate scope
        let should_remove = {
            if let Some(target_entry) = users.borrow().get(&principal) {
                // Prevent removal of owners by admins
                if matches!(target_entry.role, AuthRole::Owner)
                    && !matches!(auth_info.role, AuthRole::Owner)
                {
                    ic_cdk::trap("Only owners can remove other owners");
                }

                // Prevent self-removal
                if principal == caller {
                    ic_cdk::trap("Cannot remove yourself");
                }

                true
            } else {
                false
            }
        }; // Immutable borrow is dropped here

        // Now we can safely get a mutable borrow
        if should_remove {
            users.borrow_mut().remove(&principal);

            log_auth_action(
                AuthAction::RemoveUser,
                principal,
                Some(principal),
                caller,
                true,
                format!("User removed by {}", caller.to_text()),
            );

            format!(
                "Principal {} removed by {}",
                principal.to_text(),
                caller.to_text()
            )
        } else {
            ic_cdk::trap(&format!("User {} not found", principal.to_text()))
        }
    })
}

/// Update user role (requires Owner role)
pub fn update_user_role(principal: Principal, new_role: AuthRole) -> String {
    // Security check: prevent anonymous principal from having any role
    if principal == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot have a role");
    }

    require_role_or_higher(AuthRole::Owner); // Only owners can change roles
    let caller = ic_cdk::caller();

    AUTH_USERS.with(|users| {
        if let Some(mut auth_entry) = users.borrow().get(&principal) {
            let old_role = auth_entry.role.clone();
            auth_entry.role = new_role.clone();
            users.borrow_mut().insert(principal, auth_entry);

            log_auth_action(
                AuthAction::UpdateRole,
                principal,
                Some(principal),
                caller,
                true,
                format!("Role changed from {:?} to {:?}", old_role, new_role),
            );

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

/// Get all authorized users (requires Admin or Owner role)
pub fn get_authorized_users() -> Vec<User> {
    require_role_or_higher(AuthRole::Admin);
    let caller = ic_cdk::caller();

    log_auth_action(
        AuthAction::ViewAuditLog,
        caller,
        None,
        caller,
        true,
        "Viewed authorized users list".to_string(),
    );

    AUTH_USERS.with(|users| {
        users
            .borrow()
            .iter()
            .map(|(_, entry)| entry.clone())
            .collect()
    })
}

/// Get authentication audit log (requires Owner role)
pub fn get_auth_audit(limit: Option<u32>) -> Vec<AuthAuditEntry> {
    require_role_or_higher(AuthRole::Owner);
    let caller = ic_cdk::caller();
    let limit = limit.unwrap_or(100).min(1000) as usize;

    log_auth_action(
        AuthAction::ViewAuditLog,
        caller,
        None,
        caller,
        true,
        format!("Viewed audit log (limit: {})", limit),
    );

    AUTH_AUDIT.with(|audit| {
        let mut entries: Vec<AuthAuditEntry> = audit
            .borrow()
            .iter()
            .map(|(_, entry)| entry.clone())
            .collect();

        // Sort by timestamp (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);

        entries
    })
}

/// Log authentication action for audit trail
fn log_auth_action(
    action: AuthAction,
    principal: Principal,
    target_principal: Option<Principal>,
    performed_by: Principal,
    success: bool,
    details: String,
) {
    let audit_id = AUTH_COUNTER.with(|c| {
        let current = *c.borrow();
        *c.borrow_mut() = current + 1;
        let timestamp = ic_cdk::api::time() / 1_000_000;
        format!("audit_{}_{}", timestamp, current + 1)
    });

    let audit_entry = AuthAuditEntry {
        id: audit_id.clone(),
        timestamp: ic_cdk::api::time(),
        action,
        principal,
        target_principal,
        performed_by,
        success,
        details,
    };

    AUTH_AUDIT.with(|audit| {
        audit.borrow_mut().insert(audit_id, audit_entry);
    });
}

/// Get current caller's authentication status
pub fn get_auth_status() -> AuthInfo {
    authenticate()
}

/// Get all users as structured data
pub fn list_users() -> Vec<User> {
    get_authorized_users()
}

/// Get specific user by principal
pub fn get_user(principal: Principal) -> Option<User> {
    AUTH_USERS.with(|users| users.borrow().get(&principal))
}

// Convenience macros for common auth checks
#[macro_export]
macro_rules! require_auth {
    () => {
        $crate::auth::authenticate();
    };
}

#[macro_export]
macro_rules! require_admin {
    () => {
        $crate::auth::require_role_or_higher($crate::auth::AuthRole::Admin);
    };
}

#[macro_export]
macro_rules! require_owner {
    () => {
        $crate::auth::require_role_or_higher($crate::auth::AuthRole::Owner);
    };
}
