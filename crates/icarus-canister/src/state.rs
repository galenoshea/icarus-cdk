//! Canister state management

use crate::memory::{get_memory, MEMORY_ID_CONFIG, MEMORY_ID_TOOLS, MEMORY_ID_RESOURCES};
use ic_stable_structures::{StableBTreeMap, StableCell, Storable};
use std::cell::RefCell;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

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
}

/// State for individual resources
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ResourceState {
    pub uri: String,
    pub access_count: u64,
}

impl Storable for ServerConfig {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }
    
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = 
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 1024,
            is_fixed_size: false,
        };
}

impl Storable for ToolState {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }
    
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: ic_stable_structures::storable::Bound = 
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 512,
            is_fixed_size: false,
        };
}

impl Storable for ResourceState {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(candid::encode_one(self).unwrap())
    }
    
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
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
            config: StableCell::init(get_memory(MEMORY_ID_CONFIG), config).unwrap(),
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
    let caller = ic_cdk::caller();
    IcarusCanisterState::with(|state| {
        let owner = state.get_owner();
        if caller != owner {
            ic_cdk::trap(&format!(
                "Access denied: caller {} is not the owner {}",
                caller.to_text(),
                owner.to_text()
            ));
        }
    });
}

/// Check if the caller is the canister owner without trapping
pub fn is_owner(caller: Principal) -> bool {
    IcarusCanisterState::with(|state| {
        caller == state.get_owner()
    })
}

/// Get the current owner principal
pub fn get_owner() -> Principal {
    IcarusCanisterState::with(|state| state.get_owner())
}