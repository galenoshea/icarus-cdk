//! Authentication module for Internet Identity integration
//!
//! Handles user authentication and identity management for the bridge

#![allow(dead_code)]

use anyhow::{Context, Result};
use candid::Principal;
use ic_agent::{Agent, Identity};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub ic_url: String,
    pub ii_url: String,
    pub use_local: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            ic_url: "https://ic0.app".to_string(),
            ii_url: "https://identity.ic0.app".to_string(),
            use_local: false,
        }
    }
}

impl AuthConfig {
    pub fn local() -> Self {
        Self {
            ic_url: "http://localhost:4943".to_string(),
            ii_url: "http://localhost:4943/?canister=rdmx6-jaaaa-aaaaa-aaadq-cai".to_string(),
            use_local: true,
        }
    }
}

/// Authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub principal: Principal,
    pub session_token: Option<String>,
    pub expires_at: Option<u64>,
}

/// Authentication manager for Internet Identity
pub struct AuthManager {
    config: AuthConfig,
    agent: Option<Agent>,
    identity: Option<Arc<dyn Identity>>,
    auth_state: Option<AuthState>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            agent: None,
            identity: None,
            auth_state: None,
        }
    }

    /// Check if user is currently authenticated
    pub fn is_authenticated(&self) -> bool {
        // For development mode, having auth_state is sufficient
        // In production, we'd require both auth_state and identity
        self.auth_state.is_some()
    }

    /// Get the current user's principal
    pub fn get_principal(&self) -> Option<Principal> {
        self.auth_state.as_ref().map(|state| state.principal)
    }

    /// Get the authenticated agent for making canister calls
    pub fn get_agent(&self) -> Option<&Agent> {
        self.agent.as_ref()
    }

    /// Set the principal directly (for local development)
    pub fn set_principal(&mut self, principal: Principal) {
        self.auth_state = Some(AuthState {
            principal,
            session_token: None,
            expires_at: None,
        });
    }

    /// Initiate authentication flow
    pub async fn authenticate(&mut self) -> Result<Principal> {
        eprintln!("🔐 Starting Internet Identity authentication...");

        if self.config.use_local {
            self.authenticate_local().await
        } else {
            self.authenticate_mainnet().await
        }
    }

    /// Authenticate using local Internet Identity
    async fn authenticate_local(&mut self) -> Result<Principal> {
        eprintln!("🏠 Using local Internet Identity");

        // For local development, we'll use an anonymous identity for now
        // In a real implementation, you'd integrate with the local II canister
        let agent = Agent::builder()
            .with_url(&self.config.ic_url)
            .build()
            .context("Failed to create agent")?;

        // Fetch root key for local development
        agent
            .fetch_root_key()
            .await
            .context("Failed to fetch root key")?;

        // For now, use anonymous identity - this will be replaced with proper II flow
        let principal = Principal::anonymous();

        self.agent = Some(agent);
        self.auth_state = Some(AuthState {
            principal,
            session_token: None,
            expires_at: None,
        });

        eprintln!("✅ Authenticated as: {}", principal.to_text());
        Ok(principal)
    }

    /// Authenticate using mainnet Internet Identity
    async fn authenticate_mainnet(&mut self) -> Result<Principal> {
        eprintln!("🌐 Using mainnet Internet Identity");

        // Create agent
        let _agent = Agent::builder()
            .with_url(&self.config.ic_url)
            .build()
            .context("Failed to create agent")?;

        // For now, we'll implement a simplified flow
        // In a full implementation, this would:
        // 1. Open a browser to the II URL
        // 2. Handle the authentication callback
        // 3. Extract the identity and create an authenticated agent

        eprintln!("🚧 Mainnet authentication not yet implemented");
        eprintln!("📝 Please use local development for now with --local flag");

        anyhow::bail!("Mainnet authentication not yet implemented. Use --local for development.");
    }

    /// Logout and clear authentication state
    pub fn logout(&mut self) {
        self.agent = None;
        self.identity = None;
        self.auth_state = None;
        eprintln!("👋 Logged out successfully");
    }

    /// Check if the current session is still valid
    pub fn is_session_valid(&self) -> bool {
        if let Some(state) = &self.auth_state {
            if let Some(expires_at) = state.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now < expires_at;
            }
            // If no expiration time, assume valid
            return true;
        }
        false
    }

    /// Refresh the authentication session if needed
    pub async fn refresh_if_needed(&mut self) -> Result<()> {
        if !self.is_session_valid() {
            eprintln!("🔄 Session expired, re-authenticating...");
            self.authenticate().await?;
        }
        Ok(())
    }
}

/// Helper function to create an authenticated agent
pub async fn create_authenticated_agent(
    config: &AuthConfig,
    _principal: Principal,
) -> Result<Agent> {
    let agent = Agent::builder()
        .with_url(&config.ic_url)
        .build()
        .context("Failed to create agent")?;

    if config.use_local {
        agent
            .fetch_root_key()
            .await
            .context("Failed to fetch root key")?;
    }

    Ok(agent)
}

/// Check if Internet Identity is available
pub async fn check_ii_availability(config: &AuthConfig) -> Result<bool> {
    let url = Url::parse(&config.ii_url).context("Invalid Internet Identity URL")?;

    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}
