//! Canister state management

use crate::memory::{get_memory, MEMORY_ID_CONFIG, MEMORY_ID_TOOLS, MEMORY_ID_RESOURCES};
use ic_stable_structures::{StableBTreeMap, StableCell, Storable};
use icarus_core::protocol::IcarusServerCapabilities;
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
    pub static STATE: RefCell<Option<IcarusCanisterState>> = RefCell::new(None);
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
    
    pub fn capabilities(&self) -> IcarusServerCapabilities {
        let config = self.config.get();
        IcarusServerCapabilities {
            tools: self.tools.iter().map(|(k, _)| k).collect(),
            resources: self.resources.iter().map(|(k, _)| k).collect(),
            icarus_version: env!("CARGO_PKG_VERSION").to_string(),
            canister_id: config.canister_id,
        }
    }
}

// State should not be cloneable as it contains stable memory structures
// Use STATE.with() to access the global state instead