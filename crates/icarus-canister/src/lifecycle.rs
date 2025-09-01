//! Canister lifecycle hooks

use crate::state::{IcarusCanisterState, ServerConfig};
use candid::Principal;

/// Initialize the canister with an owner
pub fn init(owner: Principal) {
    let config = ServerConfig {
        name: "Icarus MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        canister_id: ic_cdk::id(),
        owner,
    };

    IcarusCanisterState::init(config);
}

/// Initialize the canister with the caller as owner (for backward compatibility)
pub fn init_with_caller() {
    init(ic_cdk::caller());
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
