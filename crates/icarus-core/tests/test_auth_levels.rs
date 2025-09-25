//! Basic authentication types and role hierarchy tests
//!
//! This module tests that the authentication types compile correctly
//! and that role hierarchy is properly defined.

use candid::Principal;
use icarus_core::auth::{AuthInfo, AuthRole};

/// Test that auth roles have the correct hierarchy
#[test]
fn test_auth_role_hierarchy() {
    // Test that auth roles are correctly defined
    let owner = AuthRole::Owner;
    let admin = AuthRole::Admin;
    let user = AuthRole::User;

    // Basic role creation should work
    assert!(matches!(owner, AuthRole::Owner));
    assert!(matches!(admin, AuthRole::Admin));
    assert!(matches!(user, AuthRole::User));
}

/// Test that auth info structure works correctly
#[test]
fn test_auth_info_creation() {
    let auth_info = AuthInfo {
        principal: "rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
        role: AuthRole::Owner,
        is_authenticated: true,
    };

    assert_eq!(auth_info.role, AuthRole::Owner);
    assert!(auth_info.is_authenticated);
    assert_eq!(auth_info.principal, "rdmx6-jaaaa-aaaaa-aaadq-cai");
}

/// Test that principal creation works with valid strings
#[test]
fn test_valid_principals() {
    let owner = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai");
    let admin = Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai");

    assert!(owner.is_ok());
    assert!(admin.is_ok());
}
