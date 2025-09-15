//! Client for communicating with ICP canisters
//!
//! Handles Candid encoding/decoding and HTTP calls to ICP

use anyhow::Result;
use candid::types::value::{IDLArgs, IDLValue};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_agent::Agent;
use serde::Serialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::param_mapper::ParamMapper;

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
    param_mapper: Arc<RwLock<Option<ParamMapper>>>,
}

/// Client for interacting with ICP canisters.
///
/// This includes methods for interacting with template canisters created by `icarus new`.
/// The memory-related methods (memorize, recall, list, forget) correspond to the
/// default functions generated in the canister template and allow testing the
/// MCP bridge with a standard canister interface.
#[allow(dead_code)] // These methods are part of the SDK's public API for template interaction
impl CanisterClient {
    /// Create a new canister client for local development
    pub async fn new(canister_id: Principal) -> Result<Self> {
        let agent = Agent::builder().with_url("http://localhost:4943").build()?;

        // For local development, fetch root key
        agent.fetch_root_key().await?;

        Ok(Self {
            canister_id,
            agent,
            param_mapper: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a new canister client with authentication from session (removed for now)
    pub async fn new_authenticated(canister_id: Principal) -> Result<Self> {
        // For now, just use the anonymous agent
        Self::new(canister_id).await
    }

    /// Create a new canister client with an authenticated agent
    pub fn new_with_agent(canister_id: Principal, agent: Agent) -> Self {
        Self {
            canister_id,
            agent,
            param_mapper: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if current principal is authorized to use this canister
    pub async fn check_authorization(&self) -> Result<bool> {
        let principal = self
            .agent
            .get_principal()
            .map_err(|e| anyhow::anyhow!("Failed to get principal: {}", e))?;
        let result: Vec<u8> = self
            .agent
            .query(&self.canister_id, "is_authorized")
            .with_arg(Encode!(&principal)?)
            .call()
            .await?;

        let authorized = Decode!(&result, bool)?;
        Ok(authorized)
    }

    /// Refresh tool definitions from canister
    pub async fn refresh_tools(&self) -> Result<()> {
        let tools_json = self
            .generic_call("list_tools", serde_json::json!({}), true)
            .await?;
        let mapper = ParamMapper::from_tools_list(&tools_json)?;
        *self.param_mapper.write().await = Some(mapper);
        Ok(())
    }

    /// Get canister metadata including owner information
    pub async fn get_canister_metadata(&self) -> Result<String> {
        let result: Vec<u8> = self
            .agent
            .query(&self.canister_id, "list_tools")
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
        let result: Vec<u8> = self
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
        let result: Vec<u8> = self
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

        let response: Vec<u8> = self
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

        let response: Vec<u8> = self
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

        let response: Vec<u8> = self
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

        let response: Vec<u8> = self
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
    pub async fn list_tools(&self) -> Result<String> {
        let response: Vec<u8> = self
            .agent
            .query(&self.canister_id, "list_tools")
            .with_arg(Encode!(&()).unwrap())
            .call()
            .await?;

        let metadata = Decode!(&response, String)?;
        Ok(metadata)
    }

    /// Get the current caller's principal identity (whoami)
    pub async fn whoami(&self) -> Result<String> {
        let response: Vec<u8> = self.agent.query(&self.canister_id, "whoami").call().await?;

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

        // Ensure we have tool definitions (skip if we're calling list_tools to avoid recursion)
        if method_name != "list_tools" && self.param_mapper.read().await.is_none() {
            if debug {
                eprintln!("[DEBUG] Refreshing tool definitions...");
            }
            // Directly call list_tools without using generic_call to avoid recursion
            if let Ok(response) = self
                .agent
                .query(&self.canister_id, "list_tools")
                .with_arg(Encode!(&())?)
                .call()
                .await as Result<Vec<u8>, _>
            {
                if let Ok(tools_json) = self.decode_response(response) {
                    if let Ok(mapper) = ParamMapper::from_tools_list(&tools_json) {
                        *self.param_mapper.write().await = Some(mapper);
                    }
                }
            }
            if debug && self.param_mapper.read().await.is_none() {
                eprintln!("[DEBUG] Failed to refresh tools. Using fallback encoding.");
            }
        }

        // Convert JSON args to Candid format using ParamMapper
        let candid_args = if let Some(mapper) = self.param_mapper.read().await.as_ref() {
            // Use intelligent parameter mapping
            if debug {
                eprintln!(
                    "[DEBUG] Using ParamMapper for '{}' with args: {}",
                    method_name, args
                );
            }

            mapper
                .encode_with_fallback(method_name, args.clone())
                .unwrap_or_else(|e| {
                    if debug {
                        eprintln!(
                            "[DEBUG] ParamMapper failed: {}. Using fallback encoding.",
                            e
                        );
                    }
                    // Fallback to old behavior
                    self.fallback_encode(args.clone())
                        .unwrap_or_else(|_| vec![])
                })
        } else {
            // No mapper available, use fallback encoding
            if debug {
                eprintln!("[DEBUG] No ParamMapper available, using fallback encoding");
            }
            self.fallback_encode(args)?
        };

        if debug {
            eprintln!(
                "[DEBUG] Candid args bytes: {:?} (length: {})",
                candid_args,
                candid_args.len()
            );
        }

        let response: Vec<u8> = if is_query {
            self.agent
                .query(&self.canister_id, method_name)
                .with_arg(&candid_args[..])
                .call()
                .await?
        } else {
            self.agent
                .update(&self.canister_id, method_name)
                .with_arg(&candid_args[..])
                .call_and_wait()
                .await?
        };

        // Try to decode response
        self.decode_response(response)
    }

    /// Fallback encoding when ParamMapper is not available
    fn fallback_encode(&self, args: serde_json::Value) -> Result<Vec<u8>> {
        if args.is_null() || (args.is_object() && args.as_object().unwrap().is_empty()) {
            // No arguments - still need proper Candid encoding for empty tuple
            Ok(Encode!(&())?)
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
            Ok(encoded_args)
        } else {
            // Single argument or object - convert to appropriate Candid type
            if let Some(s) = args.as_str() {
                Ok(Encode!(&s)?)
            } else if let Some(n) = args.as_u64() {
                Ok(Encode!(&n)?)
            } else if let Some(b) = args.as_bool() {
                Ok(Encode!(&b)?)
            } else {
                // Fallback to string representation
                let s = args.to_string();
                Ok(Encode!(&s)?)
            }
        }
    }

    /// Convert an IDLValue to JSON
    fn idl_to_json(&self, value: &IDLValue) -> JsonValue {
        match value {
            IDLValue::Null => json!(null),
            IDLValue::Bool(b) => json!(b),
            IDLValue::Number(s) => {
                // Try to parse as various number types
                if let Ok(n) = s.parse::<i64>() {
                    json!(n)
                } else if let Ok(n) = s.parse::<u64>() {
                    json!(n)
                } else if let Ok(n) = s.parse::<f64>() {
                    json!(n)
                } else {
                    // Fallback to string for very large numbers
                    json!(s)
                }
            }
            IDLValue::Int(i) => json!(i.to_string()),
            IDLValue::Nat(n) => json!(n.to_string()),
            IDLValue::Nat8(n) => json!(n),
            IDLValue::Nat16(n) => json!(n),
            IDLValue::Nat32(n) => json!(n),
            IDLValue::Nat64(n) => json!(n),
            IDLValue::Int8(i) => json!(i),
            IDLValue::Int16(i) => json!(i),
            IDLValue::Int32(i) => json!(i),
            IDLValue::Int64(i) => json!(i),
            IDLValue::Float32(f) => json!(f),
            IDLValue::Float64(f) => json!(f),
            IDLValue::Text(s) => json!(s),
            IDLValue::None => json!(null),
            IDLValue::Opt(boxed_value) => {
                // Opt contains a Box<IDLValue>, not an Option
                self.idl_to_json(boxed_value)
            }
            IDLValue::Vec(values) => {
                let array: Vec<JsonValue> = values.iter().map(|v| self.idl_to_json(v)).collect();
                json!(array)
            }
            IDLValue::Record(fields) => {
                let mut object = serde_json::Map::new();
                for field in fields {
                    // Field structure: id (Id(u32) or Name(String)) and val (IDLValue)
                    let key = field.id.to_string();
                    object.insert(key, self.idl_to_json(&field.val));
                }
                json!(object)
            }
            IDLValue::Variant(variant) => {
                // Variant is a Box containing a field (with id and val) and a u64 code
                let mut object = serde_json::Map::new();
                let key = variant.0.id.to_string();
                object.insert(key, self.idl_to_json(&variant.0.val));
                json!(object)
            }
            IDLValue::Principal(p) => json!(p.to_text()),
            IDLValue::Service(p) => json!(p.to_text()),
            IDLValue::Func(p, m) => json!(format!("{}::{}", p.to_text(), m)),
            IDLValue::Reserved => json!("reserved"),
            _ => json!(format!("{:?}", value)), // Fallback for any unhandled types
        }
    }

    /// Decode response from canister using dynamic Candid decoding
    fn decode_response(&self, response: Vec<u8>) -> Result<String> {
        let debug = std::env::var("ICARUS_DEBUG").is_ok();

        if debug {
            eprintln!(
                "[DEBUG] Attempting to decode response of {} bytes",
                response.len()
            );
        }

        // Try to decode as IDLArgs for universal handling
        match IDLArgs::from_bytes(&response) {
            Ok(idl_args) => {
                if debug {
                    eprintln!(
                        "[DEBUG] Successfully decoded as IDLArgs with {} values",
                        idl_args.args.len()
                    );
                }

                // Convert the first argument to JSON (most responses have a single value)
                if let Some(first_arg) = idl_args.args.first() {
                    let json_value = self.idl_to_json(first_arg);

                    // Special handling for Result variants
                    if let Some(obj) = json_value.as_object() {
                        if obj.contains_key("Ok") {
                            // This is a Result::Ok, extract the inner value
                            if let Some(ok_value) = obj.get("Ok") {
                                // If the Ok value is a string, return it directly
                                if let Some(s) = ok_value.as_str() {
                                    return Ok(s.to_string());
                                } else {
                                    // Otherwise return the JSON representation
                                    return Ok(serde_json::to_string_pretty(ok_value)?);
                                }
                            }
                        } else if obj.contains_key("Err") {
                            // This is a Result::Err, return as error
                            if let Some(err_value) = obj.get("Err") {
                                if let Some(s) = err_value.as_str() {
                                    return Err(anyhow::anyhow!("Canister error: {}", s));
                                } else {
                                    return Err(anyhow::anyhow!("Canister error: {}", err_value));
                                }
                            }
                        }
                    }

                    // For non-Result types, return the JSON representation
                    if let Some(s) = json_value.as_str() {
                        // If it's already a string, return it directly
                        Ok(s.to_string())
                    } else {
                        // Otherwise return the JSON representation
                        Ok(serde_json::to_string_pretty(&json_value)?)
                    }
                } else {
                    // No arguments in response
                    Ok("null".to_string())
                }
            }
            Err(e) => {
                if debug {
                    eprintln!("[DEBUG] Failed to decode as IDLArgs: {}", e);
                }

                Err(anyhow::anyhow!(
                    "Unable to decode canister response. The response format is not supported. Error: {}",
                    e
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    fn create_test_principal() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap()
    }

    fn create_mock_agent() -> Agent {
        // Create a mock agent for testing (will fail on actual calls)
        Agent::builder()
            .with_url("http://localhost:8000")
            .build()
            .unwrap()
    }

    #[test]
    fn test_new_with_agent() {
        let principal = create_test_principal();
        let agent = create_mock_agent();

        let client = CanisterClient::new_with_agent(principal, agent);

        assert_eq!(client.canister_id, principal);
    }

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry {
            id: "test-id".to_string(),
            content: "test content".to_string(),
            created_at: 123456789,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.content, "test content");
        assert_eq!(entry.created_at, 123456789);
        assert_eq!(entry.tags.len(), 2);
    }

    #[test]
    fn test_memory_entry_serialization() {
        let entry = MemoryEntry {
            id: "test".to_string(),
            content: "content".to_string(),
            created_at: 123,
            tags: vec!["tag".to_string()],
        };

        // Test that it can be serialized to JSON
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("content"));

        // Test that it can be deserialized back
        let deserialized: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.content, entry.content);
    }

    #[test]
    fn test_principal_parsing() {
        // Test that we can parse valid canister IDs
        let valid_principal = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai");
        assert!(valid_principal.is_ok());

        let invalid_principal = Principal::from_text("invalid-principal");
        assert!(invalid_principal.is_err());
    }

    #[test]
    fn test_client_with_different_principals() {
        let agent = create_mock_agent();
        let principal1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let principal2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        let client1 = CanisterClient::new_with_agent(principal1, agent.clone());
        let client2 = CanisterClient::new_with_agent(principal2, agent);

        assert_eq!(client1.canister_id, principal1);
        assert_eq!(client2.canister_id, principal2);
        assert_ne!(client1.canister_id, client2.canister_id);
    }

    #[test]
    fn test_memory_entry_tag_handling() {
        let entry_with_tags = MemoryEntry {
            id: "1".to_string(),
            content: "content".to_string(),
            created_at: 123,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let entry_without_tags = MemoryEntry {
            id: "2".to_string(),
            content: "content".to_string(),
            created_at: 123,
            tags: vec![],
        };

        assert_eq!(entry_with_tags.tags.len(), 2);
        assert_eq!(entry_without_tags.tags.len(), 0);
        assert!(entry_with_tags.tags.contains(&"tag1".to_string()));
    }

    #[tokio::test]
    async fn test_client_creation_without_network() {
        // Test that client creation works without actual network calls
        let principal = create_test_principal();
        let agent = create_mock_agent();
        let client = CanisterClient::new_with_agent(principal, agent);

        // Verify client properties
        assert_eq!(client.canister_id, principal);

        // Test that param_mapper is initialized
        let mapper = client.param_mapper.read().await;
        assert!(mapper.is_none()); // Should be None initially
    }
}
