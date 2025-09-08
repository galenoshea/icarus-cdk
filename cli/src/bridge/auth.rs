//! Authentication module for Internet Identity integration
//!
//! Handles user authentication and identity management for the bridge

use anyhow::{Context, Result};
use ic_agent::Agent;

/// Helper function to create an agent for a specific network
pub async fn create_agent(ic_url: &str, use_local: bool) -> Result<Agent> {
    let agent = Agent::builder()
        .with_url(ic_url)
        .build()
        .context("Failed to create agent")?;

    if use_local {
        agent
            .fetch_root_key()
            .await
            .context("Failed to fetch root key")?;
    }

    Ok(agent)
}
