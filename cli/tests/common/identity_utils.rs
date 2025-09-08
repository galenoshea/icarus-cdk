//! Identity management utilities for testing

#![allow(dead_code)]

use candid::Principal;
use ic_agent::identity::{BasicIdentity, Identity};
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Identity manager for tests
#[allow(dead_code)]
pub struct IdentityManager {
    identities: Arc<Mutex<HashMap<String, Box<dyn Identity>>>>,
    principals: Arc<Mutex<HashMap<String, Principal>>>,
}

impl IdentityManager {
    /// Create a new identity manager
    pub fn new() -> Self {
        Self {
            identities: Arc::new(Mutex::new(HashMap::new())),
            principals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new test identity with a given name
    pub fn create_identity(&self, name: &str) -> Principal {
        // Generate a new key pair
        let rng = SystemRandom::new();
        let pkcs8_bytes =
            Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key pair");
        let _key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("Failed to create key pair from PKCS8");

        // Extract the raw private key (seed) from the key pair
        // The PKCS8 format includes the 32-byte seed at a specific offset
        let pkcs8_bytes = pkcs8_bytes.as_ref();
        // PKCS8 Ed25519 format has the 32-byte seed starting at offset 16
        let seed_offset = 16;
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&pkcs8_bytes[seed_offset..seed_offset + 32]);

        let identity = BasicIdentity::from_raw_key(&seed);
        let principal = identity.sender().expect("Failed to get principal");

        // Store identity and principal
        self.identities
            .lock()
            .unwrap()
            .insert(name.to_string(), Box::new(identity));

        self.principals
            .lock()
            .unwrap()
            .insert(name.to_string(), principal);

        principal
    }

    /// Get a principal by name
    pub fn get_principal(&self, name: &str) -> Option<Principal> {
        self.principals.lock().unwrap().get(name).copied()
    }

    /// Create standard test identities
    pub fn create_standard_identities(&self) -> TestIdentities {
        TestIdentities {
            owner: self.create_identity("owner"),
            alice_admin: self.create_identity("alice_admin"),
            bob_user: self.create_identity("bob_user"),
            charlie_readonly: self.create_identity("charlie_readonly"),
            eve_unauthorized: self.create_identity("eve_unauthorized"),
            anonymous: Principal::anonymous(),
        }
    }
}

/// Standard test identities
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestIdentities {
    pub owner: Principal,
    pub alice_admin: Principal,
    pub bob_user: Principal,
    pub charlie_readonly: Principal,
    pub eve_unauthorized: Principal,
    pub anonymous: Principal,
}

impl TestIdentities {
    /// Get all authorized principals with their expected roles
    pub fn authorized_users(&self) -> Vec<(Principal, &str)> {
        vec![
            (self.owner, "Owner"),
            (self.alice_admin, "Admin"),
            (self.bob_user, "User"),
            (self.charlie_readonly, "ReadOnly"),
        ]
    }

    /// Get unauthorized principals
    pub fn unauthorized_users(&self) -> Vec<Principal> {
        vec![self.eve_unauthorized, self.anonymous]
    }
}

/// Helper to generate a random principal for testing
#[allow(dead_code)]
pub fn generate_random_principal() -> Principal {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key pair");

    // Extract the raw private key (seed) from the PKCS8 bytes
    let pkcs8_bytes = pkcs8_bytes.as_ref();
    // PKCS8 Ed25519 format has the 32-byte seed starting at offset 16
    let seed_offset = 16;
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&pkcs8_bytes[seed_offset..seed_offset + 32]);

    let identity = BasicIdentity::from_raw_key(&seed);
    identity.sender().expect("Failed to get principal")
}

/// Create a batch of random principals
#[allow(dead_code)]
pub fn generate_principals(count: usize) -> Vec<Principal> {
    (0..count).map(|_| generate_random_principal()).collect()
}

/// Mock dfx identity switching for testing
#[allow(dead_code)]
pub struct MockDfxIdentity {
    current: Arc<Mutex<String>>,
    identities: Arc<Mutex<HashMap<String, Principal>>>,
}

impl MockDfxIdentity {
    pub fn new() -> Self {
        let mut identities = HashMap::new();
        identities.insert("default".to_string(), generate_random_principal());

        Self {
            current: Arc::new(Mutex::new("default".to_string())),
            identities: Arc::new(Mutex::new(identities)),
        }
    }

    /// Switch to a different identity
    pub fn switch_identity(&self, name: &str) -> Principal {
        let mut identities = self.identities.lock().unwrap();

        // Create identity if it doesn't exist
        if !identities.contains_key(name) {
            identities.insert(name.to_string(), generate_random_principal());
        }

        *self.current.lock().unwrap() = name.to_string();
        identities[name]
    }

    /// Get current identity
    pub fn current_identity(&self) -> (String, Principal) {
        let current = self.current.lock().unwrap().clone();
        let principal = self.identities.lock().unwrap()[&current];
        (current, principal)
    }

    /// Add a specific principal for an identity
    pub fn add_identity(&self, name: &str, principal: Principal) {
        self.identities
            .lock()
            .unwrap()
            .insert(name.to_string(), principal);
    }
}
