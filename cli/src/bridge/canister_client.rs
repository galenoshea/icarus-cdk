//! Client for communicating with ICP canisters
//!
//! Handles Candid encoding/decoding and HTTP calls to ICP

use anyhow::Result;
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_agent::Agent;
use serde::Serialize;

/// A memory entry stored in the canister
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}

/// Client for interacting with ICP canisters
#[derive(Clone)]
pub struct CanisterClient {
    canister_id: Principal,
    agent: Agent,
}

impl CanisterClient {
    /// Create a new canister client for local development
    pub async fn new(canister_id: Principal) -> Result<Self> {
        let agent = Agent::builder().with_url("http://localhost:4943").build()?;

        // For local development, fetch root key
        agent.fetch_root_key().await?;

        Ok(Self { canister_id, agent })
    }

    /// Create a new canister client with authentication from session (removed for now)
    pub async fn new_authenticated(canister_id: Principal) -> Result<Self> {
        // For now, just use the anonymous agent
        Self::new(canister_id).await
    }

    /// Create a new canister client with an authenticated agent
    pub fn new_with_agent(canister_id: Principal, agent: Agent) -> Self {
        Self { canister_id, agent }
    }

    /// Check if current principal is authorized to use this canister
    pub async fn check_authorization(&self) -> Result<bool> {
        let principal = self
            .agent
            .get_principal()
            .map_err(|e| anyhow::anyhow!("Failed to get principal: {}", e))?;
        let result = self
            .agent
            .query(&self.canister_id, "is_authorized")
            .with_arg(Encode!(&principal)?)
            .call()
            .await?;

        let authorized = Decode!(&result, bool)?;
        Ok(authorized)
    }

    /// Get canister metadata including owner information
    pub async fn get_canister_metadata(&self) -> Result<String> {
        let result = self
            .agent
            .query(&self.canister_id, "get_metadata")
            .call()
            .await?;

        // The canister returns Result<String, Error>, we need to handle both cases
        match Decode!(&result, Result<String, String>) {
            Ok(Ok(metadata)) => Ok(metadata),
            Ok(Err(error)) => Err(anyhow::anyhow!("Canister error: {}", error)),
            Err(decode_error) => {
                // Fallback: try to decode as plain string
                match Decode!(&result, String) {
                    Ok(metadata) => Ok(metadata),
                    Err(_) => Err(anyhow::anyhow!(
                        "Failed to decode metadata: {}",
                        decode_error
                    )),
                }
            }
        }
    }

    /// Add a new owner to the canister (requires current user to be an owner)
    pub async fn add_owner(&self, new_owner: Principal) -> Result<()> {
        let result = self
            .agent
            .update(&self.canister_id, "add_owner")
            .with_arg(Encode!(&new_owner)?)
            .call_and_wait()
            .await?;

        match Decode!(&result, Result<(), String>) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(anyhow::anyhow!("Failed to add owner: {}", error)),
            Err(decode_error) => Err(anyhow::anyhow!(
                "Failed to decode response: {}",
                decode_error
            )),
        }
    }

    /// List all owners of the canister
    pub async fn list_owners(&self) -> Result<(Principal, Vec<Principal>)> {
        let result = self
            .agent
            .query(&self.canister_id, "list_owners")
            .call()
            .await?;

        match Decode!(&result, Result<(Principal, Vec<Principal>), String>) {
            Ok(Ok(owners)) => Ok(owners),
            Ok(Err(error)) => Err(anyhow::anyhow!("Failed to list owners: {}", error)),
            Err(decode_error) => Err(anyhow::anyhow!("Failed to decode owners: {}", decode_error)),
        }
    }

    /// Store a new memory with optional tags
    pub async fn memorize(&self, content: String, tags: Option<Vec<String>>) -> Result<String> {
        let args = Encode!(&content, &tags)?;

        let response = self
            .agent
            .update(&self.canister_id, "memorize")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: Result<String, String> = Decode!(&response, Result<String, String>)?;

        match result {
            Ok(id) => Ok(id),
            Err(e) => Err(anyhow::anyhow!("Canister error: {}", e)),
        }
    }

    /// Recall memories by tag
    pub async fn recall(&self, tag: String) -> Result<Vec<MemoryEntry>> {
        let args = Encode!(&tag)?;

        let response = self
            .agent
            .query(&self.canister_id, "recall")
            .with_arg(args)
            .call()
            .await?;

        let memories = Decode!(&response, Vec<MemoryEntry>)?;
        Ok(memories)
    }

    /// List all memories with optional limit
    pub async fn list(&self, limit: Option<u64>) -> Result<Vec<MemoryEntry>> {
        let args = Encode!(&limit)?;

        let response = self
            .agent
            .query(&self.canister_id, "list")
            .with_arg(args)
            .call()
            .await?;

        let memories = Decode!(&response, Vec<MemoryEntry>)?;
        Ok(memories)
    }

    /// Delete a memory by ID
    pub async fn forget(&self, id: String) -> Result<bool> {
        let args = Encode!(&id)?;

        let response = self
            .agent
            .update(&self.canister_id, "forget")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: Result<bool, String> = Decode!(&response, Result<bool, String>)?;

        match result {
            Ok(success) => Ok(success),
            Err(e) => Err(anyhow::anyhow!("Canister error: {}", e)),
        }
    }

    /// Get canister metadata
    pub async fn get_metadata(&self) -> Result<String> {
        let response = self
            .agent
            .query(&self.canister_id, "get_metadata")
            .with_arg(Encode!(&()).unwrap())
            .call()
            .await?;

        let metadata = Decode!(&response, String)?;
        Ok(metadata)
    }

    /// Get the current caller's principal identity (whoami)
    pub async fn whoami(&self) -> Result<String> {
        let response = self.agent.query(&self.canister_id, "whoami").call().await?;

        let result: Result<String, String> = Decode!(&response, Result<String, String>)?;

        match result {
            Ok(whoami_info) => Ok(whoami_info),
            Err(e) => Err(anyhow::anyhow!("Canister error: {}", e)),
        }
    }

    /// Generic call to any canister method with JSON arguments
    pub async fn generic_call(
        &self,
        method_name: &str,
        args: serde_json::Value,
        is_query: bool,
    ) -> Result<String> {
        let debug = std::env::var("ICARUS_DEBUG").is_ok();

        if debug {
            eprintln!(
                "[DEBUG] Calling canister method '{}' with args: {}",
                method_name, args
            );
        }

        // Convert JSON args to Candid format
        let candid_args =
            if args.is_null() || (args.is_object() && args.as_object().unwrap().is_empty()) {
                // No arguments - still need proper Candid encoding for empty tuple
                Encode!(&())?
            } else if args.is_array() {
                // Arguments as array - encode each element
                let array = args.as_array().unwrap();
                let mut encoded_args = Vec::new();
                for arg in array {
                    // For now, assume string arguments - can be enhanced later
                    if let Some(s) = arg.as_str() {
                        encoded_args.extend(Encode!(&s)?);
                    } else if let Some(n) = arg.as_u64() {
                        encoded_args.extend(Encode!(&n)?);
                    } else if let Some(b) = arg.as_bool() {
                        encoded_args.extend(Encode!(&b)?);
                    } else {
                        // Fallback to string representation
                        let s = arg.to_string();
                        encoded_args.extend(Encode!(&s)?);
                    }
                }
                encoded_args
            } else {
                // Single argument or object - convert to appropriate Candid type
                if let Some(s) = args.as_str() {
                    Encode!(&s)?
                } else if let Some(n) = args.as_u64() {
                    Encode!(&n)?
                } else if let Some(b) = args.as_bool() {
                    Encode!(&b)?
                } else {
                    // Fallback to string representation
                    let s = args.to_string();
                    Encode!(&s)?
                }
            };

        if debug {
            eprintln!(
                "[DEBUG] Candid args bytes: {:?} (length: {})",
                candid_args,
                candid_args.len()
            );
            eprintln!("[DEBUG] Args is_empty: {}", candid_args.is_empty());
        }

        let response = if is_query {
            // Always use with_arg since we now always have proper Candid encoding
            self.agent
                .query(&self.canister_id, method_name)
                .with_arg(&candid_args[..])
                .call()
                .await?
        } else {
            // Always use with_arg since we now always have proper Candid encoding
            self.agent
                .update(&self.canister_id, method_name)
                .with_arg(&candid_args[..])
                .call_and_wait()
                .await?
        };

        // Try to decode as Result<String, String> first (common pattern)
        if let Ok(result) = Decode!(&response, Result<String, String>) {
            match result {
                Ok(success) => Ok(success),
                Err(e) => Err(anyhow::anyhow!("Canister error: {}", e)),
            }
        } else if let Ok(string_result) = Decode!(&response, String) {
            // Direct string response
            Ok(string_result)
        } else {
            // Fallback: try to decode as bytes or return error
            Err(anyhow::anyhow!(
                "Unable to decode canister response. The response format is not supported."
            ))
        }
    }
}
