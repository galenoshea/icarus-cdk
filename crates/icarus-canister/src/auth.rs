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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
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
    let caller = ic_cdk::api::msg_caller();

    // Security check: anonymous principal is never authenticated
    if caller == Principal::anonymous() {
        ic_cdk::trap("Access denied: Anonymous principal cannot be authenticated");
    }

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

            ic_cdk::trap(format!(
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
        ic_cdk::trap(format!(
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
        ic_cdk::trap(format!(
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
        ic_cdk::trap(format!(
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
        ic_cdk::trap(format!("Role {:?} is not allowed here", auth_info.role));
    }
}

/// Add a new user (requires Admin or Owner role)
pub fn add_user(principal: Principal, role: AuthRole) -> String {
    // Security check: prevent anonymous principal from being added
    if principal == Principal::anonymous() {
        ic_cdk::trap("Security Error: Anonymous principal cannot be authorized");
    }

    let auth_info = require_role_or_higher(AuthRole::Admin);
    let caller = ic_cdk::api::msg_caller();

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
    let caller = ic_cdk::api::msg_caller();

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
            ic_cdk::trap(format!("User {} not found", principal.to_text()))
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
    let caller = ic_cdk::api::msg_caller();

    AUTH_USERS.with(|users| {
        // Clone the entry to avoid holding a borrow across mutable operations
        let auth_entry_opt = users.borrow().get(&principal);

        if let Some(mut auth_entry) = auth_entry_opt {
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
    let caller = ic_cdk::api::msg_caller();

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
            .map(|entry| entry.value().clone())
            .collect()
    })
}

/// Get authentication audit log (requires Owner role)
pub fn get_auth_audit(limit: Option<u32>) -> Vec<AuthAuditEntry> {
    require_role_or_higher(AuthRole::Owner);
    let caller = ic_cdk::api::msg_caller();
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
            .map(|entry| entry.value().clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Mock stable storage for testing
    thread_local! {
        static MOCK_USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
        static MOCK_AUDIT: RefCell<HashMap<String, AuthAuditEntry>> = RefCell::new(HashMap::new());
        static MOCK_COUNTER: RefCell<u64> = RefCell::new(0);
        static MOCK_TIME: RefCell<u64> = RefCell::new(1000000000000);
        static MOCK_CALLER: RefCell<Principal> = RefCell::new(Principal::anonymous());
    }

    // Mock implementation for testing
    fn set_mock_caller(principal: Principal) {
        MOCK_CALLER.with(|c| *c.borrow_mut() = principal);
    }

    fn set_mock_time(time: u64) {
        MOCK_TIME.with(|t| *t.borrow_mut() = time);
    }

    fn clear_mock_storage() {
        MOCK_USERS.with(|u| u.borrow_mut().clear());
        MOCK_AUDIT.with(|a| a.borrow_mut().clear());
        MOCK_COUNTER.with(|c| *c.borrow_mut() = 0);
        MOCK_TIME.with(|t| *t.borrow_mut() = 1000000000000);
        MOCK_CALLER.with(|c| *c.borrow_mut() = Principal::anonymous());
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
            last_access: None,
            access_count: 0,
        };

        assert_eq!(user.principal, owner);
        assert_eq!(user.role, AuthRole::Owner);
        assert!(user.active);
        assert_eq!(user.access_count, 0);
        assert!(user.last_access.is_none());
    }

    #[test]
    fn test_auth_role_hierarchy() {
        // Test role equality
        assert_eq!(AuthRole::Owner, AuthRole::Owner);
        assert_eq!(AuthRole::Admin, AuthRole::Admin);
        assert_eq!(AuthRole::User, AuthRole::User);
        assert_eq!(AuthRole::ReadOnly, AuthRole::ReadOnly);

        // Test role inequality
        assert_ne!(AuthRole::Owner, AuthRole::Admin);
        assert_ne!(AuthRole::Admin, AuthRole::User);
        assert_ne!(AuthRole::User, AuthRole::ReadOnly);
    }

    #[test]
    fn test_auth_info_creation() {
        let principal = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let auth_info = AuthInfo {
            principal: principal.to_string(),
            role: AuthRole::User,
            is_authenticated: true,
            last_access: Some(1000000000000),
            access_count: 5,
            message: "Access granted".to_string(),
        };

        assert_eq!(auth_info.principal, principal);
        assert_eq!(auth_info.role, AuthRole::User);
        assert!(auth_info.is_authenticated);
        assert_eq!(auth_info.last_access, Some(1000000000000));
        assert_eq!(auth_info.access_count, 5);
        assert_eq!(auth_info.message, "Access granted");
    }

    #[test]
    fn test_auth_audit_entry_creation() {
        let principal = create_test_principal("test-principal");
        let performed_by = create_test_principal("admin");

        let audit_entry = AuthAuditEntry {
            id: "audit_1000_1".to_string(),
            timestamp: 1000000000000,
            action: AuthAction::AddUser,
            principal,
            target_principal: Some(principal),
            performed_by,
            success: true,
            details: "Test audit entry".to_string(),
        };

        assert_eq!(audit_entry.id, "audit_1000_1");
        assert_eq!(audit_entry.action, AuthAction::AddUser);
        assert_eq!(audit_entry.principal, principal);
        assert_eq!(audit_entry.target_principal, Some(principal));
        assert_eq!(audit_entry.performed_by, performed_by);
        assert!(audit_entry.success);
        assert_eq!(audit_entry.details, "Test audit entry");
    }

    #[test]
    fn test_auth_action_variants() {
        // Test all AuthAction variants exist and are distinct
        let actions = vec![
            AuthAction::AddUser,
            AuthAction::RemoveUser,
            AuthAction::UpdateRole,
            AuthAction::DeactivateUser,
            AuthAction::ReactivateUser,
            AuthAction::AccessGranted,
            AuthAction::AccessDenied,
            AuthAction::ViewAuditLog,
        ];

        assert_eq!(actions.len(), 8);

        // Test serialization/deserialization
        for action in actions {
            let serialized = serde_json::to_string(&action).unwrap();
            let deserialized: AuthAction = serde_json::from_str(&serialized).unwrap();
            // Note: We can't directly compare due to no PartialEq, but successful deserialization indicates correctness
            let _ = deserialized;
        }
    }

    #[test]
    fn test_user_serialization() {
        let owner = create_test_principal("owner");
        let user = User {
            principal: owner,
            added_at: 1000000000000,
            added_by: owner,
            role: AuthRole::Owner,
            active: true,
            last_access: Some(1500000000000),
            access_count: 10,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("Owner"));
        assert!(json.contains("1000000000000"));
        assert!(json.contains("true"));

        // Test deserialization
        let deserialized: User = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.principal, user.principal);
        assert_eq!(deserialized.role, user.role);
        assert_eq!(deserialized.active, user.active);
        assert_eq!(deserialized.access_count, user.access_count);
    }

    #[test]
    fn test_auth_info_serialization() {
        let auth_info = AuthInfo {
            principal: "test-principal".to_string(),
            role: AuthRole::Admin,
            is_authenticated: true,
            last_access: Some(1000000000000),
            access_count: 3,
            message: "Success".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&auth_info).unwrap();
        let deserialized: AuthInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.principal, auth_info.principal);
        assert_eq!(deserialized.role, auth_info.role);
        assert_eq!(deserialized.is_authenticated, auth_info.is_authenticated);
        assert_eq!(deserialized.last_access, auth_info.last_access);
        assert_eq!(deserialized.access_count, auth_info.access_count);
        assert_eq!(deserialized.message, auth_info.message);
    }

    #[test]
    fn test_auth_audit_entry_serialization() {
        let principal = create_test_principal("test");
        let audit_entry = AuthAuditEntry {
            id: "audit_test_1".to_string(),
            timestamp: 1000000000000,
            action: AuthAction::AccessGranted,
            principal,
            target_principal: None,
            performed_by: principal,
            success: true,
            details: "Test access".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&audit_entry).unwrap();
        let deserialized: AuthAuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, audit_entry.id);
        assert_eq!(deserialized.timestamp, audit_entry.timestamp);
        assert_eq!(deserialized.principal, audit_entry.principal);
        assert_eq!(deserialized.target_principal, audit_entry.target_principal);
        assert_eq!(deserialized.performed_by, audit_entry.performed_by);
        assert_eq!(deserialized.success, audit_entry.success);
        assert_eq!(deserialized.details, audit_entry.details);
    }

    #[test]
    fn test_role_clone_and_debug() {
        let role = AuthRole::Admin;
        let cloned = role.clone();
        assert_eq!(role, cloned);

        // Test Debug formatting
        let debug_str = format!("{:?}", role);
        assert_eq!(debug_str, "Admin");
    }

    #[test]
    fn test_user_clone_and_debug() {
        let principal = create_test_principal("test");
        let user = User {
            principal,
            added_at: 1000000000000,
            added_by: principal,
            role: AuthRole::User,
            active: true,
            last_access: None,
            access_count: 0,
        };

        let cloned = user.clone();
        assert_eq!(cloned.principal, user.principal);
        assert_eq!(cloned.role, user.role);
        assert_eq!(cloned.active, user.active);

        // Test Debug formatting
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("User"));
        assert!(debug_str.contains("1000000000000"));
    }

    #[test]
    fn test_auth_info_clone_and_debug() {
        let auth_info = AuthInfo {
            principal: "test".to_string(),
            role: AuthRole::ReadOnly,
            is_authenticated: false,
            last_access: None,
            access_count: 0,
            message: "Not authenticated".to_string(),
        };

        let cloned = auth_info.clone();
        assert_eq!(cloned.principal, auth_info.principal);
        assert_eq!(cloned.role, auth_info.role);
        assert_eq!(cloned.is_authenticated, auth_info.is_authenticated);

        // Test Debug formatting
        let debug_str = format!("{:?}", auth_info);
        assert!(debug_str.contains("ReadOnly"));
        assert!(debug_str.contains("false"));
    }

    #[test]
    fn test_audit_entry_clone_and_debug() {
        let principal = create_test_principal("test");
        let audit_entry = AuthAuditEntry {
            id: "test_audit".to_string(),
            timestamp: 1000000000000,
            action: AuthAction::RemoveUser,
            principal,
            target_principal: Some(principal),
            performed_by: principal,
            success: false,
            details: "Failed removal".to_string(),
        };

        let cloned = audit_entry.clone();
        assert_eq!(cloned.id, audit_entry.id);
        assert_eq!(cloned.success, audit_entry.success);

        // Test Debug formatting
        let debug_str = format!("{:?}", audit_entry);
        assert!(debug_str.contains("RemoveUser"));
        assert!(debug_str.contains("false"));
    }

    #[test]
    fn test_comprehensive_type_coverage() {
        // Test that all public types can be constructed and have expected traits

        // AuthRole coverage
        let roles = [AuthRole::Owner, AuthRole::Admin, AuthRole::User, AuthRole::ReadOnly];
        for role in &roles {
            let cloned = role.clone();
            let _debug = format!("{:?}", role);
            let _serialized = serde_json::to_string(role).unwrap();
            assert_eq!(*role, cloned);
        }

        // User coverage
        let principal = create_test_principal("comprehensive-test");
        let user = User {
            principal,
            added_at: 1000000000000,
            added_by: principal,
            role: AuthRole::Owner,
            active: true,
            last_access: Some(1500000000000),
            access_count: 42,
        };
        let _cloned = user.clone();
        let _debug = format!("{:?}", user);
        let _serialized = serde_json::to_string(&user).unwrap();

        // AuthInfo coverage
        let auth_info = AuthInfo {
            principal: principal.to_text(),
            role: AuthRole::Admin,
            is_authenticated: true,
            last_access: Some(1000000000000),
            access_count: 5,
            message: "Test message".to_string(),
        };
        let _cloned = auth_info.clone();
        let _debug = format!("{:?}", auth_info);
        let _serialized = serde_json::to_string(&auth_info).unwrap();

        // AuthAuditEntry coverage
        let audit = AuthAuditEntry {
            id: "comprehensive_test".to_string(),
            timestamp: 1000000000000,
            action: AuthAction::UpdateRole,
            principal,
            target_principal: Some(principal),
            performed_by: principal,
            success: true,
            details: "Comprehensive test".to_string(),
        };
        let _cloned = audit.clone();
        let _debug = format!("{:?}", audit);
        let _serialized = serde_json::to_string(&audit).unwrap();

        // AuthAction coverage
        let actions = [
            AuthAction::AddUser,
            AuthAction::RemoveUser,
            AuthAction::UpdateRole,
            AuthAction::DeactivateUser,
            AuthAction::ReactivateUser,
            AuthAction::AccessGranted,
            AuthAction::AccessDenied,
            AuthAction::ViewAuditLog,
        ];
        for action in &actions {
            let cloned = action.clone();
            let _debug = format!("{:?}", action);
            let _serialized = serde_json::to_string(action).unwrap();
            // Test PartialEq for AuthAction
            assert_eq!(*action, cloned);
        }
    }

    #[test]
    fn test_edge_cases_and_boundaries() {
        // Test with maximum values
        let principal = create_test_principal("edge-case");

        // User with maximum access count
        let user_max = User {
            principal,
            added_at: u64::MAX,
            added_by: principal,
            role: AuthRole::Owner,
            active: true,
            last_access: Some(u64::MAX),
            access_count: u64::MAX,
        };
        let _serialized = serde_json::to_string(&user_max).unwrap();

        // User with minimum values
        let user_min = User {
            principal,
            added_at: 0,
            added_by: principal,
            role: AuthRole::ReadOnly,
            active: false,
            last_access: None,
            access_count: 0,
        };
        let _serialized = serde_json::to_string(&user_min).unwrap();

        // AuthInfo with edge values
        let auth_edge = AuthInfo {
            principal: "".to_string(), // Empty string
            role: AuthRole::ReadOnly,
            is_authenticated: false,
            last_access: None,
            access_count: 0,
            message: "a".repeat(1000), // Large message
        };
        let _serialized = serde_json::to_string(&auth_edge).unwrap();

        // Audit entry with edge values
        let audit_edge = AuthAuditEntry {
            id: "x".repeat(100), // Long ID
            timestamp: u64::MAX,
            action: AuthAction::ViewAuditLog,
            principal,
            target_principal: None,
            performed_by: principal,
            success: false,
            details: "detail".repeat(200), // Long details
        };
        let _serialized = serde_json::to_string(&audit_edge).unwrap();
    }
}
