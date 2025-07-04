//! Canister lifecycle hooks

use crate::state::{IcarusCanisterState, ServerConfig};

/// Initialize the canister
pub fn init() {
    let config = ServerConfig {
        name: "Icarus MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        canister_id: ic_cdk::id(),
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
    init();
}