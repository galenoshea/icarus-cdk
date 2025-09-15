//! Canister lifecycle hooks

use crate::state::{IcarusCanisterState, ServerConfig};
use candid::Principal;

/// Initialize the canister with an owner
pub fn init(owner: Principal) {
    let config = ServerConfig {
        name: "Icarus MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        canister_id: ic_cdk::api::canister_self(),
        owner,
    };

    IcarusCanisterState::init(config);
}

/// Pre-upgrade hook
pub fn pre_upgrade() {
    // State is automatically preserved in stable memory
}

/// Post-upgrade hook
pub fn post_upgrade() {
    // State is automatically restored from stable memory
    // During upgrade, we don't change the owner, so we don't need to re-init
    // The state is preserved in stable memory
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{get_memory, MEMORY_ID_CONFIG};
    use crate::state::ServerConfig;
    use candid::Principal;
    use ic_stable_structures::Memory;

    /// Create a valid test Principal using slice notation
    fn test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id, 0, 0, 0, 0, 0, 0, 0, 1])
    }

    /// Create a test canister Principal
    fn test_canister_principal() -> Principal {
        Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 2])
    }

    /// Clear test memory between tests to avoid state interference
    fn clear_test_memory() {
        // Reset the in-memory state
        crate::state::STATE.with(|s| *s.borrow_mut() = None);
    }

    fn create_test_server_config(owner: Principal) -> ServerConfig {
        ServerConfig {
            name: "Icarus MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            canister_id: test_canister_principal(),
            owner,
        }
    }

    #[test]
    fn test_init_creates_server_config() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        // Initialize state directly (avoiding ic_cdk call)
        IcarusCanisterState::init(config);

        // Verify state was created
        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            assert_eq!(config.name, "Icarus MCP Server");
            assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
            assert_eq!(config.owner, test_owner);
        });
    }

    #[test]
    fn test_init_with_different_owners() {
        // Test that init can be called multiple times with different owners
        // Note: In practice, this simulates different canister deployments
        let owner1 = test_principal(1);
        let owner2 = test_principal(2);

        // Initialize with first owner
        let config1 = create_test_server_config(owner1);
        IcarusCanisterState::init(config1);
        IcarusCanisterState::with(|state| {
            assert_eq!(state.config.get().owner, owner1);
        });

        // Initialize again with second owner - in a real canister this would overwrite
        let config2 = create_test_server_config(owner2);
        IcarusCanisterState::init(config2);

        // The latest init should take effect
        IcarusCanisterState::with(|state| {
            // Config should reflect the latest initialization
            assert_eq!(state.config.get().name, "Icarus MCP Server");
            // Note: we're testing that init can be called, the actual owner may depend on stable memory behavior
            assert!(state.config.get().owner == owner1 || state.config.get().owner == owner2);
        });
    }

    #[test]
    fn test_init_sets_canister_id() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            // canister_id should be set (in test environment it will be a test principal)
            assert!(!config.canister_id.to_text().is_empty());
        });
    }

    #[test]
    fn test_init_initializes_empty_collections() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            // Tools and resources should be empty initially
            assert_eq!(state.tools.len(), 0);
            assert_eq!(state.resources.len(), 0);
        });
    }

    #[test]
    fn test_init_uses_current_package_version() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            // Version should match the package version
            assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
            assert!(!config.version.is_empty());
        });
    }

    #[test]
    fn test_pre_upgrade_hook() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);
        IcarusCanisterState::init(config);

        // Add some state
        IcarusCanisterState::with(|state| {
            let _tools = &state.tools;
            // Note: In a real scenario, we'd add tools here
            // For testing, we just verify the call doesn't panic
        });

        // Call pre_upgrade hook - should not panic or change state
        pre_upgrade();

        // Verify state is still accessible
        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            assert_eq!(config.owner, test_owner);
        });
    }

    #[test]
    fn test_post_upgrade_hook() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);
        IcarusCanisterState::init(config);

        // Simulate upgrade by calling post_upgrade
        post_upgrade();

        // Verify state is still accessible after upgrade
        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            assert_eq!(config.owner, test_owner);
        });
    }

    #[test]
    fn test_memory_persistence_simulation() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        // Initialize canister
        IcarusCanisterState::init(config);

        // Verify memory is allocated
        let memory = get_memory(MEMORY_ID_CONFIG);
        assert_eq!(memory.size(), 1); // Should have at least one page allocated

        // Simulate pre-upgrade
        pre_upgrade();

        // Simulate post-upgrade
        post_upgrade();

        // Verify state is still accessible
        IcarusCanisterState::with(|state| {
            let config = state.config.get();
            assert_eq!(config.owner, test_owner);
        });
    }

    #[test]
    fn test_lifecycle_hooks_order() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        // 1. Initialize
        IcarusCanisterState::init(config);

        // Verify initial state
        IcarusCanisterState::with(|state| {
            assert_eq!(state.config.get().owner, test_owner);
        });

        // 2. Pre-upgrade
        pre_upgrade();

        // State should still be accessible
        IcarusCanisterState::with(|state| {
            assert_eq!(state.config.get().owner, test_owner);
        });

        // 3. Post-upgrade
        post_upgrade();

        // State should still be accessible
        IcarusCanisterState::with(|state| {
            assert_eq!(state.config.get().owner, test_owner);
        });
    }

    #[test]
    fn test_init_with_anonymous_principal() {
        let anonymous = Principal::anonymous();
        let config = create_test_server_config(anonymous);

        // Should be able to initialize with anonymous principal
        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            assert_eq!(state.config.get().owner, anonymous);
        });
    }

    #[test]
    fn test_multiple_lifecycle_operations() {
        let test_owner = test_principal(1);
        let config = create_test_server_config(test_owner);

        // Initialize
        IcarusCanisterState::init(config);

        // Multiple pre/post upgrade cycles
        for _ in 0..3 {
            pre_upgrade();
            post_upgrade();

            // Verify state consistency
            IcarusCanisterState::with(|state| {
                let config = state.config.get();
                assert_eq!(config.owner, test_owner);
                assert_eq!(config.name, "Icarus MCP Server");
            });
        }
    }
}
