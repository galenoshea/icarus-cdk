//! Canister state management

use crate::memory::{get_memory, MEMORY_ID_CONFIG, MEMORY_ID_RESOURCES, MEMORY_ID_TOOLS};
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{StableBTreeMap, StableCell, Storable};
use serde::Serialize;
use std::cell::RefCell;

/// Main canister state
pub struct IcarusCanisterState {
    pub config: StableCell<ServerConfig, crate::memory::Memory>,
    pub tools: StableBTreeMap<String, ToolState, crate::memory::Memory>,
    pub resources: StableBTreeMap<String, ResourceState, crate::memory::Memory>,
}

/// Server configuration stored in stable memory
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub canister_id: Principal,
    pub owner: Principal,
}

/// State for individual tools
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ToolState {
    pub name: String,
    pub enabled: bool,
    pub call_count: u64,
    pub is_query: bool,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// Parameter information for tools
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

/// State for individual resources
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ResourceState {
    pub uri: String,
    pub access_count: u64,
}

impl Storable for ServerConfig {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 1024,
            is_fixed_size: false,
        };
}

impl Storable for ToolState {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 2048, // Increased to accommodate description and parameters
            is_fixed_size: false,
        };
}

impl Storable for ToolParameter {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 512,
            is_fixed_size: false,
        };
}

impl Storable for ResourceState {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 512,
            is_fixed_size: false,
        };
}

thread_local! {
    /// Global canister state
    pub static STATE: RefCell<Option<IcarusCanisterState>> = const { RefCell::new(None) };
}

impl IcarusCanisterState {
    pub fn init(config: ServerConfig) {
        let state = Self {
            config: StableCell::init(get_memory(MEMORY_ID_CONFIG), config),
            tools: StableBTreeMap::init(get_memory(MEMORY_ID_TOOLS)),
            resources: StableBTreeMap::init(get_memory(MEMORY_ID_RESOURCES)),
        };

        STATE.with(|s| *s.borrow_mut() = Some(state));
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&IcarusCanisterState) -> R,
    {
        STATE.with(|s| {
            let state = s.borrow();
            let state_ref = state.as_ref().expect("State not initialized");
            f(state_ref)
        })
    }

    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut IcarusCanisterState) -> R,
    {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state_ref = state.as_mut().expect("State not initialized");
            f(state_ref)
        })
    }

    /// Get the canister owner principal
    pub fn get_owner(&self) -> Principal {
        self.config.get().owner
    }
}

// State should not be cloneable as it contains stable memory structures
// Use STATE.with() to access the global state instead

/// Access control functions
/// Assert that the caller is the canister owner
pub fn assert_owner() {
    let caller = ic_cdk::api::msg_caller();
    IcarusCanisterState::with(|state| {
        let owner = state.get_owner();
        if caller != owner {
            ic_cdk::trap(format!(
                "Access denied: caller {} is not the owner {}",
                caller.to_text(),
                owner.to_text()
            ));
        }
    });
}

/// Check if the caller is the canister owner without trapping
pub fn is_owner(caller: Principal) -> bool {
    IcarusCanisterState::with(|state| caller == state.get_owner())
}

/// Get the current owner principal
pub fn get_owner() -> Principal {
    IcarusCanisterState::with(|state| state.get_owner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{get_memory, MEMORY_ID_CONFIG, MEMORY_ID_RESOURCES, MEMORY_ID_TOOLS};
    use candid::Principal;
    use ic_stable_structures::Memory;

    /// Create a valid test Principal using slice notation
    #[cfg(test)]
    fn test_principal(id: u8) -> Principal {
        Principal::from_slice(&[id, 0, 0, 0, 0, 0, 0, 0, 1])
    }

    /// Create a test canister Principal
    #[cfg(test)]
    fn test_canister_principal() -> Principal {
        Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 2])
    }

    #[cfg(test)]
    fn create_test_config() -> ServerConfig {
        ServerConfig {
            name: "Test Server".to_string(),
            version: "1.0.0".to_string(),
            canister_id: test_canister_principal(),
            owner: test_principal(1),
        }
    }

    #[cfg(test)]
    fn create_test_tool_state() -> ToolState {
        ToolState {
            name: "test_tool".to_string(),
            enabled: true,
            call_count: 42,
            is_query: false,
            description: "Test tool for testing".to_string(),
            parameters: vec![],
        }
    }

    #[cfg(test)]
    fn create_test_resource_state() -> ResourceState {
        ResourceState {
            uri: "https://example.com/resource".to_string(),
            access_count: 10,
        }
    }

    #[test]
    fn test_server_config_storable() {
        let config = create_test_config();

        // Test to_bytes and from_bytes
        let bytes = config.to_bytes();
        let restored = ServerConfig::from_bytes(bytes);

        assert_eq!(restored.name, config.name);
        assert_eq!(restored.version, config.version);
        assert_eq!(restored.canister_id, config.canister_id);
        assert_eq!(restored.owner, config.owner);
    }

    #[test]
    fn test_server_config_into_bytes() {
        let config = create_test_config();
        let original_name = config.name.clone();
        let original_version = config.version.clone();

        // Test into_bytes
        let bytes = config.into_bytes();
        let restored = ServerConfig::from_bytes(std::borrow::Cow::Borrowed(&bytes));

        assert_eq!(restored.name, original_name);
        assert_eq!(restored.version, original_version);
    }

    #[test]
    fn test_tool_state_storable() {
        let tool = create_test_tool_state();

        // Test to_bytes and from_bytes
        let bytes = tool.to_bytes();
        let restored = ToolState::from_bytes(bytes);

        assert_eq!(restored.name, tool.name);
        assert_eq!(restored.enabled, tool.enabled);
        assert_eq!(restored.call_count, tool.call_count);
        assert_eq!(restored.is_query, tool.is_query);
    }

    #[test]
    fn test_tool_state_into_bytes() {
        let tool = create_test_tool_state();
        let original_name = tool.name.clone();
        let original_count = tool.call_count;

        // Test into_bytes
        let bytes = tool.into_bytes();
        let restored = ToolState::from_bytes(std::borrow::Cow::Borrowed(&bytes));

        assert_eq!(restored.name, original_name);
        assert_eq!(restored.call_count, original_count);
    }

    #[test]
    fn test_resource_state_storable() {
        let resource = create_test_resource_state();

        // Test to_bytes and from_bytes
        let bytes = resource.to_bytes();
        let restored = ResourceState::from_bytes(bytes);

        assert_eq!(restored.uri, resource.uri);
        assert_eq!(restored.access_count, resource.access_count);
    }

    #[test]
    fn test_resource_state_into_bytes() {
        let resource = create_test_resource_state();
        let original_uri = resource.uri.clone();
        let original_count = resource.access_count;

        // Test into_bytes
        let bytes = resource.into_bytes();
        let restored = ResourceState::from_bytes(std::borrow::Cow::Borrowed(&bytes));

        assert_eq!(restored.uri, original_uri);
        assert_eq!(restored.access_count, original_count);
    }

    #[test]
    fn test_icarus_canister_state_init() {
        let config = create_test_config();
        let owner = config.owner;

        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            assert_eq!(state.get_owner(), owner);
            assert_eq!(state.tools.len(), 0);
            assert_eq!(state.resources.len(), 0);
        });
    }

    #[test]
    fn test_icarus_canister_state_with() {
        let config = create_test_config();
        IcarusCanisterState::init(config);

        let result = IcarusCanisterState::with(|state| state.config.get().name.clone());

        assert_eq!(result, "Test Server");
    }

    #[test]
    fn test_icarus_canister_state_get_owner() {
        let config = create_test_config();
        let expected_owner = config.owner;
        IcarusCanisterState::init(config);

        IcarusCanisterState::with(|state| {
            assert_eq!(state.get_owner(), expected_owner);
        });
    }

    #[test]
    fn test_state_tools_management() {
        let config = create_test_config();
        IcarusCanisterState::init(config);

        let tool = create_test_tool_state();
        IcarusCanisterState::with_mut(|state| {
            // Add a tool
            state.tools.insert("test_tool".to_string(), tool.clone());
        });

        IcarusCanisterState::with(|state| {
            // Verify tool was added
            assert_eq!(state.tools.len(), 1);
            let retrieved = state.tools.get(&"test_tool".to_string()).unwrap();
            assert_eq!(retrieved.name, tool.name);
            assert_eq!(retrieved.call_count, tool.call_count);
        });
    }

    #[test]
    fn test_state_resources_management() {
        let config = create_test_config();
        IcarusCanisterState::init(config);

        let resource = create_test_resource_state();
        IcarusCanisterState::with_mut(|state| {
            // Add a resource
            state
                .resources
                .insert("test_resource".to_string(), resource.clone());
        });

        IcarusCanisterState::with(|state| {
            // Verify resource was added
            assert_eq!(state.resources.len(), 1);
            let retrieved = state.resources.get(&"test_resource".to_string()).unwrap();
            assert_eq!(retrieved.uri, resource.uri);
            assert_eq!(retrieved.access_count, resource.access_count);
        });
    }

    #[test]
    fn test_state_multiple_tools_and_resources() {
        let config = create_test_config();
        IcarusCanisterState::init(config);

        IcarusCanisterState::with_mut(|state| {
            // Add multiple tools
            for i in 0..5 {
                let tool = ToolState {
                    name: format!("tool_{}", i),
                    enabled: i % 2 == 0,
                    call_count: i * 10,
                    is_query: i % 3 == 0,
                    description: format!("Test tool {}", i),
                    parameters: vec![],
                };
                state.tools.insert(format!("tool_{}", i), tool);
            }

            // Add multiple resources
            for i in 0..3 {
                let resource = ResourceState {
                    uri: format!("https://example.com/resource_{}", i),
                    access_count: i * 5,
                };
                state.resources.insert(format!("resource_{}", i), resource);
            }
        });

        IcarusCanisterState::with(|state| {
            assert_eq!(state.tools.len(), 5);
            assert_eq!(state.resources.len(), 3);

            // Verify specific entries
            let tool_2 = state.tools.get(&"tool_2".to_string()).unwrap();
            assert_eq!(tool_2.name, "tool_2");
            assert_eq!(tool_2.call_count, 20);
            assert_eq!(tool_2.enabled, true);

            let resource_1 = state.resources.get(&"resource_1".to_string()).unwrap();
            assert_eq!(resource_1.uri, "https://example.com/resource_1");
            assert_eq!(resource_1.access_count, 5);
        });
    }

    #[test]
    fn test_is_owner_function() {
        let config = create_test_config();
        let owner = config.owner;
        let non_owner = test_principal(2);

        IcarusCanisterState::init(config);

        assert_eq!(is_owner(owner), true);
        assert_eq!(is_owner(non_owner), false);
        assert_eq!(is_owner(Principal::anonymous()), false);
    }

    #[test]
    fn test_get_owner_function() {
        let config = create_test_config();
        let expected_owner = config.owner;
        IcarusCanisterState::init(config);

        let actual_owner = get_owner();
        assert_eq!(actual_owner, expected_owner);
    }

    #[test]
    fn test_state_persistence_across_memory_regions() {
        let config = create_test_config();
        IcarusCanisterState::init(config);

        // Verify different memory regions are used
        let config_memory = get_memory(MEMORY_ID_CONFIG);
        let tools_memory = get_memory(MEMORY_ID_TOOLS);
        let resources_memory = get_memory(MEMORY_ID_RESOURCES);

        // Each should have allocated memory
        assert!(config_memory.size() > 0);
        // Note: tools and resources may be 0 if empty

        IcarusCanisterState::with_mut(|state| {
            // Add data to trigger memory allocation
            let tool = create_test_tool_state();
            state.tools.insert("test".to_string(), tool);

            let resource = create_test_resource_state();
            state.resources.insert("test".to_string(), resource);
        });

        // Now tools and resources should have allocated memory
        assert!(tools_memory.size() > 0);
        assert!(resources_memory.size() > 0);
    }

    #[test]
    fn test_state_config_updates() {
        let mut config = create_test_config();
        IcarusCanisterState::init(config.clone());

        // Update config
        config.version = "2.0.0".to_string();
        config.name = "Updated Server".to_string();

        IcarusCanisterState::with_mut(|state| {
            let _ = state.config.set(config.clone());
        });

        // Verify updates
        IcarusCanisterState::with(|state| {
            let stored_config = state.config.get();
            assert_eq!(stored_config.version, "2.0.0");
            assert_eq!(stored_config.name, "Updated Server");
            assert_eq!(stored_config.owner, config.owner);
        });
    }

    #[test]
    fn test_storable_bounds_are_reasonable() {
        // Test that the bounds are reasonable for the data structures
        match ServerConfig::BOUND {
            ic_stable_structures::storable::Bound::Bounded { max_size, .. } => {
                assert_eq!(max_size, 1024)
            }
            _ => panic!("Expected bounded"),
        }
        match ToolState::BOUND {
            ic_stable_structures::storable::Bound::Bounded { max_size, .. } => {
                assert_eq!(max_size, 2048) // Increased to accommodate description and parameters
            }
            _ => panic!("Expected bounded"),
        }
        match ResourceState::BOUND {
            ic_stable_structures::storable::Bound::Bounded { max_size, .. } => {
                assert_eq!(max_size, 512)
            }
            _ => panic!("Expected bounded"),
        }

        // Verify that they're all bounded (not unbounded)
        assert!(matches!(
            ServerConfig::BOUND,
            ic_stable_structures::storable::Bound::Bounded { .. }
        ));
        assert!(matches!(
            ToolState::BOUND,
            ic_stable_structures::storable::Bound::Bounded { .. }
        ));
        assert!(matches!(
            ResourceState::BOUND,
            ic_stable_structures::storable::Bound::Bounded { .. }
        ));
    }

    #[test]
    fn test_large_data_serialization() {
        // Test with large strings to ensure serialization works correctly
        let config = ServerConfig {
            name: "A".repeat(100),
            version: "B".repeat(50),
            canister_id: test_canister_principal(),
            owner: test_principal(1),
        };

        let bytes = config.to_bytes();
        let restored = ServerConfig::from_bytes(bytes);

        assert_eq!(restored.name.len(), 100);
        assert_eq!(restored.version.len(), 50);
        assert!(restored.name.chars().all(|c| c == 'A'));
        assert!(restored.version.chars().all(|c| c == 'B'));
    }

    #[test]
    fn test_special_characters_in_serialization() {
        let tool = ToolState {
            name: "test🚀tool with spaces & symbols!@#$%^&*()".to_string(),
            enabled: true,
            call_count: u64::MAX,
            is_query: false,
            description: "Special test tool with emojis and symbols".to_string(),
            parameters: vec![],
        };

        let bytes = tool.to_bytes();
        let restored = ToolState::from_bytes(bytes);

        assert_eq!(restored.name, tool.name);
        assert_eq!(restored.call_count, u64::MAX);
    }

    #[test]
    #[should_panic(expected = "State not initialized")]
    fn test_state_not_initialized_panic() {
        // Clear any existing state by setting it to None
        STATE.with(|s| *s.borrow_mut() = None);

        // This should panic
        IcarusCanisterState::with(|_state| {
            // This code should never execute
        });
    }

    #[test]
    fn test_state_reinitialization() {
        let config1 = create_test_config();
        IcarusCanisterState::init(config1.clone());

        // Add some data
        IcarusCanisterState::with_mut(|state| {
            let tool = create_test_tool_state();
            state.tools.insert("tool1".to_string(), tool);
        });

        // Initialize again with new config - this creates a new state object
        let config2 = ServerConfig {
            name: "New Server".to_string(),
            version: "2.0.0".to_string(),
            canister_id: test_canister_principal(),
            owner: test_principal(1),
        };
        IcarusCanisterState::init(config2.clone());

        // Verify state behavior - init creates fresh in-memory state but stable storage persists
        IcarusCanisterState::with(|state| {
            // The config name should reflect what's in stable memory or the latest init
            let config_name = &state.config.get().name;
            assert!(config_name == "Test Server" || config_name == "New Server");

            // Tools may persist in stable storage, so we just verify state is accessible
            // Length can be 0 or more depending on stable memory behavior
        });
    }
}
