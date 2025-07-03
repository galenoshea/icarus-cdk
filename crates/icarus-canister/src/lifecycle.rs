//! Canister lifecycle hooks

use crate::state::{IcarusCanisterState, ServerConfig};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade};

/// Initialize the canister
#[init]
pub fn init() {
    let config = ServerConfig {
        name: "Icarus MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        canister_id: ic_cdk::id(),
    };
    
    IcarusCanisterState::init(config);
}

/// Pre-upgrade hook
#[pre_upgrade]
pub fn pre_upgrade() {
    // State is automatically preserved in stable memory
}

/// Post-upgrade hook
#[post_upgrade]
pub fn post_upgrade() {
    // State is automatically restored from stable memory
    init();
}