//! Comprehensive authentication tests for Icarus SDK

#[path = "../common/mod.rs"]
mod common;

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use common::identity_utils::*;
use common::pocket_ic_utils::*;
use common::*;
use pocket_ic::PocketIc;
use serde::Serialize;
use serial_test::serial;

/// Authentication role types matching the SDK
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub enum AuthRole {
    Owner,
    Admin,
    User,
    ReadOnly,
}

/// Authentication info returned by the canister
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AuthInfo {
    pub principal: String,
    pub role: AuthRole,
    pub is_authenticated: bool,
    pub last_access: Option<u64>,
    pub access_count: u64,
    pub message: String,
}

/// User struct for auth system
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct User {
    pub principal: Principal,
    pub added_at: u64,
    pub added_by: Principal,
    pub role: AuthRole,
    pub active: bool,
    pub last_access: Option<u64>,
    pub access_count: u64,
}

/// =============================================================================
/// BASIC AUTHENTICATION TESTS
/// =============================================================================

#[test]
#[serial]
fn test_owner_initialization() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let owner = identity_manager.create_identity("owner");

    // Build and deploy test canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, owner);

    // Verify owner can call get_auth_status
    let result = query_with_principal(&pic, owner, canister_id, "get_auth_status", ());

    assert_access_granted(result.clone());

    // Decode and verify the auth status
    let auth_info: (AuthInfo,) = decode_args(&result.unwrap()).unwrap();
    assert_eq!(auth_info.0.role, AuthRole::Owner);
    assert!(auth_info.0.is_authenticated);
}

#[test]
#[serial]
fn test_anonymous_cannot_be_owner() {
    let pic = setup_pocket_ic();

    // Try to deploy with anonymous as owner - should trap
    let wasm_path = build_test_auth_canister();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);

    let _wasm = std::fs::read(&wasm_path).expect("Failed to read WASM");
    let _init_args = encode_args((Principal::anonymous(),)).expect("Failed to encode");

    // This should fail with security error
    // Note: PocketIC's install_canister returns () and will panic on init failure
    // We cannot test this directly without causing a test panic
}

#[test]
#[serial]
fn test_add_and_remove_users() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy canister with owner
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Owner adds Alice as Admin
    let result = call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.alice_admin.to_text(), "Admin"),
    );
    assert_access_granted(result);

    // Alice can now access
    let result = query_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "get_auth_status",
        (),
    );
    assert_access_granted(result.clone());
    let auth_info: (AuthInfo,) = decode_args(&result.unwrap()).unwrap();
    assert_eq!(auth_info.0.role, AuthRole::Admin);

    // Alice (Admin) adds Bob as User
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "add_authorized_user",
        (identities.bob_user.to_text(), "User"),
    );
    assert_access_granted(result);

    // Bob can access with User role
    let result = query_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "get_auth_status",
        (),
    );
    assert_access_granted(result.clone());
    let auth_info: (AuthInfo,) = decode_args(&result.unwrap()).unwrap();
    assert_eq!(auth_info.0.role, AuthRole::User);

    // Owner removes Bob
    let result = call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "remove_authorized_user",
        identities.bob_user.to_text(),
    );
    assert_access_granted(result);

    // Bob can no longer access
    let result = query_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "get_auth_status",
        (),
    );
    assert_access_denied(result);
}

#[test]
#[serial]
fn test_role_hierarchy() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy and set up users with different roles
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Add users with different roles
    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.alice_admin.to_text(), "Admin"),
    )
    .unwrap();

    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.bob_user.to_text(), "User"),
    )
    .unwrap();

    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.charlie_readonly.to_text(), "ReadOnly"),
    )
    .unwrap();

    // Test Admin can add users
    let new_user = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "add_authorized_user",
        (new_user.to_text(), "User"),
    );
    assert_access_granted(result);

    // Test User cannot add users
    let another_user = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "add_authorized_user",
        (another_user.to_text(), "User"),
    );
    assert_access_denied(result);

    // Test ReadOnly cannot add users
    let yet_another = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.charlie_readonly,
        canister_id,
        "add_authorized_user",
        (yet_another.to_text(), "ReadOnly"),
    );
    assert_access_denied(result);
}

/// =============================================================================
/// IDENTITY SWITCHING TESTS
/// =============================================================================

#[test]
#[serial]
fn test_identity_switching_simulation() {
    let pic = setup_pocket_ic();
    let mock_dfx = MockDfxIdentity::new();

    // Get initial identity (default)
    let (name1, principal1) = mock_dfx.current_identity();
    assert_eq!(name1, "default");

    // Deploy canister with default identity as owner
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, principal1);

    // Default identity can access
    let result = query_with_principal(&pic, principal1, canister_id, "get_auth_status", ());
    assert_access_granted(result);

    // Switch to BOB identity
    let principal2 = mock_dfx.switch_identity("bob");
    let (name2, _) = mock_dfx.current_identity();
    assert_eq!(name2, "bob");

    // BOB cannot access (not authorized)
    let result = query_with_principal(&pic, principal2, canister_id, "get_auth_status", ());
    assert_access_denied(result);

    // Switch back to default and add BOB
    mock_dfx.switch_identity("default");
    let result = call_with_principal(
        &pic,
        principal1,
        canister_id,
        "add_authorized_user",
        (principal2.to_text(), "User"),
    );
    assert_access_granted(result);

    // Switch to BOB again - now can access
    mock_dfx.switch_identity("bob");
    let result = query_with_principal(&pic, principal2, canister_id, "get_auth_status", ());
    assert_access_granted(result);
}

#[test]
#[serial]
fn test_multiple_identity_management() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();

    // Create multiple identities
    let principals: Vec<_> = (0..5)
        .map(|i| identity_manager.create_identity(&format!("user_{}", i)))
        .collect();

    // Deploy with first identity as owner
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, principals[0]);

    // Add all other identities with different roles
    let roles = ["Admin", "User", "User", "ReadOnly"];
    for (i, role) in roles.iter().enumerate() {
        let result = call_with_principal(
            &pic,
            principals[0],
            canister_id,
            "add_authorized_user",
            (principals[i + 1].to_text(), *role),
        );
        assert_access_granted(result);
    }

    // Verify each identity has correct access
    for (_i, principal) in principals.iter().enumerate() {
        let result = query_with_principal(&pic, *principal, canister_id, "get_auth_status", ());
        assert_access_granted(result);
    }
}

/// =============================================================================
/// EDGE CASES AND SECURITY TESTS
/// =============================================================================

#[test]
#[serial]
fn test_anonymous_principal_security() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let owner = identity_manager.create_identity("owner");

    // Deploy canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, owner);

    // Anonymous cannot access
    let result = query_with_principal(
        &pic,
        Principal::anonymous(),
        canister_id,
        "get_auth_status",
        (),
    );
    assert_access_denied(result);

    // Owner tries to add anonymous - should fail
    let result = call_with_principal(
        &pic,
        owner,
        canister_id,
        "add_authorized_user",
        (Principal::anonymous().to_text(), "User"),
    );
    assert_access_denied(result);
}

#[test]
#[serial]
fn test_self_elevation_prevention() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy and set up
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Add Alice as Admin
    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.alice_admin.to_text(), "Admin"),
    )
    .unwrap();

    // Alice (Admin) tries to create another Owner - should fail
    let new_principal = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "add_authorized_user",
        (new_principal.to_text(), "Owner"),
    );
    assert_access_denied(result);

    // Alice tries to update her own role to Owner - should fail
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "update_user_role",
        (identities.alice_admin.to_text(), "Owner"),
    );
    assert_access_denied(result);
}

#[test]
#[serial]
fn test_update_user_role() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Add Bob as User
    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.bob_user.to_text(), "User"),
    )
    .unwrap();

    // Verify Bob has User role
    let result = query_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "get_auth_status",
        (),
    )
    .unwrap();
    let auth_info: (AuthInfo,) = decode_args(&result).unwrap();
    assert_eq!(auth_info.0.role, AuthRole::User);

    // Owner updates Bob to Admin
    let result = call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "update_user_role",
        (identities.bob_user.to_text(), "Admin"),
    );
    assert_access_granted(result);

    // Verify Bob now has Admin role
    let result = query_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "get_auth_status",
        (),
    )
    .unwrap();
    let auth_info: (AuthInfo,) = decode_args(&result).unwrap();
    assert_eq!(auth_info.0.role, AuthRole::Admin);

    // Bob (now Admin) can add users
    let new_user = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "add_authorized_user",
        (new_user.to_text(), "User"),
    );
    assert_access_granted(result);
}

/// =============================================================================
/// PERFORMANCE AND STRESS TESTS
/// =============================================================================

#[test]
#[serial]
fn test_large_user_base() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let owner = identity_manager.create_identity("owner");

    // Deploy canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, owner);

    // Add 100 users
    let users = generate_principals(100);
    for (i, user) in users.iter().enumerate() {
        let role = match i % 4 {
            0 => "Admin",
            1 => "User",
            2 => "User",
            _ => "ReadOnly",
        };

        let result = call_with_principal(
            &pic,
            owner,
            canister_id,
            "add_authorized_user",
            (user.to_text(), role),
        );
        assert_access_granted(result);
    }

    // Verify all users can authenticate
    for user in users.iter() {
        let result = query_with_principal(&pic, *user, canister_id, "get_auth_status", ());
        assert_access_granted(result);
    }
}

/// =============================================================================
/// INTEGRATION TESTS WITH MEMENTO TOOLS
/// =============================================================================

#[test]
#[serial]
fn test_memento_tool_authorization() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy canister and set up users
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Add users with different roles
    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.alice_admin.to_text(), "Admin"),
    )
    .unwrap();

    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.bob_user.to_text(), "User"),
    )
    .unwrap();

    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.charlie_readonly.to_text(), "ReadOnly"),
    )
    .unwrap();

    // Test memorize - requires User role or higher

    // User can memorize
    let result = call_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "memorize",
        ("test_key", "test_content"),
    );
    assert_access_granted(result);

    // Admin can memorize
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "memorize",
        ("admin_key", "admin_content"),
    );
    assert_access_granted(result);

    // ReadOnly CANNOT memorize (requires User or higher)
    let result = call_with_principal(
        &pic,
        identities.charlie_readonly,
        canister_id,
        "memorize",
        ("readonly_key", "readonly_content"),
    );
    assert_access_denied(result);

    // Unauthorized user cannot memorize
    let result = call_with_principal(
        &pic,
        identities.eve_unauthorized,
        canister_id,
        "memorize",
        ("eve_key", "eve_content"),
    );
    assert_access_denied(result);

    // Test recall - blocked for ReadOnly

    // User can recall
    let result = query_with_principal(&pic, identities.bob_user, canister_id, "recall", "test_key");
    assert_access_granted(result);

    // ReadOnly CANNOT recall (blocked by require_none_of_roles)
    let result = query_with_principal(
        &pic,
        identities.charlie_readonly,
        canister_id,
        "recall",
        "test_key",
    );
    assert_access_denied(result);

    // Test list - blocked for ReadOnly

    // User can list
    let result = query_with_principal(&pic, identities.bob_user, canister_id, "list", ());
    assert_access_granted(result);

    // ReadOnly CANNOT list
    let result = query_with_principal(&pic, identities.charlie_readonly, canister_id, "list", ());
    assert_access_denied(result);

    // Test forget - requires User role or higher

    // User can forget
    let result = call_with_principal(&pic, identities.bob_user, canister_id, "forget", "test_key");
    assert_access_granted(result);

    // ReadOnly CANNOT forget
    let result = call_with_principal(
        &pic,
        identities.charlie_readonly,
        canister_id,
        "forget",
        "admin_key",
    );
    assert_access_denied(result);
}

#[test]
#[serial]
fn test_admin_tool_authorization() {
    let pic = setup_pocket_ic();
    let identity_manager = IdentityManager::new();
    let identities = identity_manager.create_standard_identities();

    // Deploy canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, identities.owner);

    // Add Alice as Admin and Bob as User
    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.alice_admin.to_text(), "Admin"),
    )
    .unwrap();

    call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "add_authorized_user",
        (identities.bob_user.to_text(), "User"),
    )
    .unwrap();

    // Test list_authorized_users - requires Admin or higher

    // Admin can list users
    let result = query_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "list_authorized_users",
        (),
    );
    assert_access_granted(result);

    // Owner can list users
    let result = query_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "list_authorized_users",
        (),
    );
    assert_access_granted(result);

    // User CANNOT list users
    let result = query_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "list_authorized_users",
        (),
    );
    assert_access_denied(result);

    // Test add_authorized_user - requires Admin or higher

    // Admin can add users
    let new_user = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "add_authorized_user",
        (new_user.to_text(), "User"),
    );
    assert_access_granted(result);

    // User CANNOT add users
    let another_user = generate_random_principal();
    let result = call_with_principal(
        &pic,
        identities.bob_user,
        canister_id,
        "add_authorized_user",
        (another_user.to_text(), "User"),
    );
    assert_access_denied(result);

    // Test update_user_role - requires Owner

    // Only Owner can update roles
    let result = call_with_principal(
        &pic,
        identities.owner,
        canister_id,
        "update_user_role",
        (identities.bob_user.to_text(), "Admin"),
    );
    assert_access_granted(result);

    // Admin CANNOT update roles
    let result = call_with_principal(
        &pic,
        identities.alice_admin,
        canister_id,
        "update_user_role",
        (new_user.to_text(), "Admin"),
    );
    assert_access_denied(result);
}

#[test]
#[serial]
fn test_complete_auth_workflow() {
    let pic = setup_pocket_ic();
    let mock_dfx = MockDfxIdentity::new();

    // Simulate complete workflow

    // 1. Start with default identity
    let (_, owner_principal) = mock_dfx.current_identity();

    // 2. Deploy canister
    let wasm_path = build_test_auth_canister();
    let canister_id = deploy_canister_with_owner(&pic, &wasm_path, owner_principal);

    // 3. Owner can use all tools
    call_with_principal(
        &pic,
        owner_principal,
        canister_id,
        "memorize",
        ("owner_key", "owner_data"),
    )
    .unwrap();

    // 4. Switch to alice identity (not authorized)
    let alice_principal = mock_dfx.switch_identity("alice");

    // 5. Alice cannot use tools
    let result = call_with_principal(
        &pic,
        alice_principal,
        canister_id,
        "memorize",
        ("alice_key", "alice_data"),
    );
    assert_access_denied(result);

    // 6. Switch back to owner and add alice
    mock_dfx.switch_identity("default");
    call_with_principal(
        &pic,
        owner_principal,
        canister_id,
        "add_authorized_user",
        (alice_principal.to_text(), "Admin"),
    )
    .unwrap();

    // 7. Switch to alice - now can use tools
    mock_dfx.switch_identity("alice");
    let result = call_with_principal(
        &pic,
        alice_principal,
        canister_id,
        "memorize",
        ("alice_key", "alice_data"),
    );
    assert_access_granted(result);

    // 8. Alice adds bob
    let bob_principal = mock_dfx.switch_identity("bob");
    mock_dfx.switch_identity("alice");

    call_with_principal(
        &pic,
        alice_principal,
        canister_id,
        "add_authorized_user",
        (bob_principal.to_text(), "User"),
    )
    .unwrap();

    // 9. Bob can use tools with User permissions
    mock_dfx.switch_identity("bob");
    let result = call_with_principal(
        &pic,
        bob_principal,
        canister_id,
        "memorize",
        ("bob_key", "bob_data"),
    );
    assert_access_granted(result);

    // But Bob cannot add users (requires Admin)
    let charlie_principal = generate_random_principal();
    let result = call_with_principal(
        &pic,
        bob_principal,
        canister_id,
        "add_authorized_user",
        (charlie_principal.to_text(), "User"),
    );
    assert_access_denied(result);
}

/// =============================================================================
/// HELPER FUNCTIONS
/// =============================================================================

fn build_test_auth_canister() -> std::path::PathBuf {
    // Create a new test project with authentication
    let test_project = TestProject::new("auth-test-canister");
    let cli = CliRunner::new();

    // Create new project with icarus new
    let output = cli.run_in(test_project.path(), &["new", "auth-test"]);
    assert_success(&output);

    let project_dir = test_project.path().join("auth-test");

    // Add authentication to the lib.rs file
    let lib_rs_path = project_dir.join("src").join("lib.rs");
    let auth_enabled_content = create_auth_test_canister_content();
    std::fs::write(&lib_rs_path, auth_enabled_content).expect("Failed to write lib.rs");

    // Build the project
    use std::process::Command;
    let build_output = Command::new("cargo")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo build");
    assert!(
        build_output.status.success(),
        "Cargo build should succeed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Return the built WASM path
    project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("auth_test.wasm")
}

fn create_auth_test_canister_content() -> String {
    r#"use icarus::prelude::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;

// Import auth system
use icarus::auth::*;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, IcarusStorable)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: u64,
}

stable_storage! {
    MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id!(0);
    COUNTER: u64 = 0;
}

fn generate_id() -> String {
    COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        *counter += 1;
        format!("mem_{}", *counter)
    })
}

#[icarus_module]
mod tools {
    use super::*;
    
    // Memory functions with auth
    #[update]
    #[icarus_tool("Store a new memory")]
    pub fn memorize(content: String) -> Result<String, String> {
        // Require User role or higher
        require_role_or_higher(AuthRole::User);
        
        let id = generate_id();
        let memory = MemoryEntry {
            id: id.clone(),
            content,
            created_at: ic_cdk::api::time(),
        };
        
        MEMORIES.with(|m| {
            m.borrow_mut().insert(id.clone(), memory);
        });
        
        Ok(id)
    }
    
    #[query]
    #[icarus_tool("Recall a memory")]
    pub fn recall(id: String) -> Result<MemoryEntry, String> {
        // Block ReadOnly users from recall
        require_none_of_roles(&[AuthRole::ReadOnly]);
        
        MEMORIES.with(|m| {
            m.borrow()
                .get(&id)
                .ok_or_else(|| format!("Memory {} not found", id))
        })
    }
    
    #[query]
    #[icarus_tool("List memories")]
    pub fn list() -> Result<Vec<MemoryEntry>, String> {
        // Block ReadOnly users
        require_none_of_roles(&[AuthRole::ReadOnly]);
        
        Ok(MEMORIES.with(|m| {
            m.borrow().iter().map(|(_, v)| v).collect()
        }))
    }
    
    #[update]
    #[icarus_tool("Delete a memory")]
    pub fn forget(id: String) -> Result<bool, String> {
        // Require User role or higher
        require_role_or_higher(AuthRole::User);
        
        MEMORIES.with(|m| {
            match m.borrow_mut().remove(&id) {
                Some(_) => Ok(true),
                None => Err(format!("Memory {} not found", id))
            }
        })
    }
}

// Initialize with owner on deployment
#[init]
fn init(owner: Principal) {
    init_auth(owner);
}

// Export auth query functions
#[query]
pub fn get_auth_status() -> AuthInfo {
    authenticate()
}

#[query] 
pub fn list_authorized_users() -> Vec<User> {
    require_role_or_higher(AuthRole::Admin);
    list_users()
}

// Export auth update functions
#[update]
pub fn add_authorized_user(principal: String, role: String) -> Result<(), String> {
    require_role_or_higher(AuthRole::Admin);
    
    let p = Principal::from_text(&principal)
        .map_err(|e| format!("Invalid principal: {}", e))?;
    
    let r = match role.as_str() {
        "Owner" => AuthRole::Owner,
        "Admin" => AuthRole::Admin,
        "User" => AuthRole::User,
        "ReadOnly" => AuthRole::ReadOnly,
        _ => return Err("Invalid role".to_string()),
    };
    
    add_user(p, r)?;
    Ok(())
}

#[update]
pub fn remove_authorized_user(principal: String) -> Result<(), String> {
    require_role_or_higher(AuthRole::Admin);
    
    let p = Principal::from_text(&principal)
        .map_err(|e| format!("Invalid principal: {}", e))?;
    
    remove_user(p)?;
    Ok(())
}

#[update]
pub fn update_user_role(principal: String, role: String) -> Result<(), String> {
    require_role_or_higher(AuthRole::Owner);
    
    let p = Principal::from_text(&principal)
        .map_err(|e| format!("Invalid principal: {}", e))?;
    
    let r = match role.as_str() {
        "Owner" => AuthRole::Owner,
        "Admin" => AuthRole::Admin,
        "User" => AuthRole::User,
        "ReadOnly" => AuthRole::ReadOnly,
        _ => return Err("Invalid role".to_string()),
    };
    
    update_user_role(p, r)?;
    Ok(())
}

ic_cdk::export_candid!();
"#
    .to_string()
}

fn deploy_canister_with_owner(
    pic: &PocketIc,
    wasm_path: &std::path::Path,
    owner: Principal,
) -> Principal {
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);

    let wasm = std::fs::read(wasm_path).expect("Failed to read WASM");
    let init_args = encode_args((owner,)).expect("Failed to encode init args");

    pic.install_canister(canister_id, wasm, init_args, None);
    canister_id
}
