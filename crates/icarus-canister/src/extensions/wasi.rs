//! WASI polyfill extension for Icarus canisters
//!
//! This module provides a WASI polyfill extension that enables canisters to use
//! libraries that depend on WebAssembly System Interface (WASI) functions.
//! The extension handles initialization of the ic-wasi-polyfill library with
//! proper memory management and deterministic configuration.

use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};
use icarus_core::extensions::{InitError, InitRequirements, InitializationExtension};
use std::cell::RefCell;

/// Configuration for WASI polyfill initialization
#[derive(Debug, Clone)]
pub struct WasiConfig {
    /// Random seed for WASI operations (if None, uses canister ID)
    pub seed: Option<[u8; 32]>,
    /// Environment variables to provide to WASI
    pub env_vars: Vec<(String, String)>,
    /// Memory range for WASI operations (default: 200..210)
    pub memory_range: std::ops::Range<u8>,
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self {
            seed: None,
            env_vars: Vec::new(),
            memory_range: 200..210,
        }
    }
}

impl WasiConfig {
    /// Create a new WASI configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom random seed
    pub fn with_seed(mut self, seed: [u8; 32]) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Add an environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Set the memory range for WASI operations
    pub fn with_memory_range(mut self, range: std::ops::Range<u8>) -> Self {
        self.memory_range = range;
        self
    }
}

/// WASI polyfill extension for canisters
pub struct WasiExtension {
    _memory_manager: RefCell<MemoryManager<DefaultMemoryImpl>>,
    config: WasiConfig,
}

impl InitializationExtension for WasiExtension {
    type Config = WasiConfig;

    const NAME: &'static str = "wasi";

    fn init_requirements() -> InitRequirements {
        InitRequirements::new()
            .with_memory_range(200..210)
            .with_random_seed()
    }

    fn initialize(config: Self::Config) -> Result<Self, InitError> {
        // Create memory manager for WASI operations
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());

        // Generate deterministic seed from canister ID if not provided
        let seed = config.seed.unwrap_or_else(|| {
            let canister_id = ic_cdk::api::canister_self();
            let id_bytes = canister_id.as_slice();
            let mut seed = [0u8; 32];

            // Use canister ID bytes to create deterministic seed
            for (i, &byte) in id_bytes.iter().take(32).enumerate() {
                seed[i] = byte;
            }

            // Fill remaining bytes with a pattern based on the first bytes
            for i in id_bytes.len()..32 {
                seed[i] = seed[i % id_bytes.len()].wrapping_add(i as u8);
            }

            seed
        });

        // Convert environment variables to the format expected by ic-wasi-polyfill
        let env_vars: Vec<(&str, &str)> = config
            .env_vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        // Initialize WASI polyfill with memory manager
        ic_wasi_polyfill::init_with_memory_manager(
            &seed,
            &env_vars,
            &memory_manager,
            config.memory_range.clone(),
        );

        ic_cdk::println!(
            "WASI polyfill initialized with memory range {:?} and {} environment variables",
            config.memory_range,
            config.env_vars.len()
        );

        Ok(Self {
            _memory_manager: RefCell::new(memory_manager),
            config,
        })
    }

    fn pre_upgrade(&self) -> Result<(), InitError> {
        // WASI state is managed by ic-wasi-polyfill internally
        // No explicit cleanup needed
        Ok(())
    }

    fn post_upgrade(&mut self) -> Result<(), InitError> {
        // Re-initialize WASI polyfill after upgrade
        Self::initialize(self.config.clone())?;
        Ok(())
    }
}

impl WasiExtension {
    /// Get the memory range used by this WASI extension
    pub fn memory_range(&self) -> &std::ops::Range<u8> {
        &self.config.memory_range
    }

    /// Get the environment variables configured for WASI
    pub fn env_vars(&self) -> &[(String, String)] {
        &self.config.env_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_config_default() {
        let config = WasiConfig::default();
        assert_eq!(config.memory_range, 200..210);
        assert!(config.env_vars.is_empty());
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_wasi_config_builder() {
        let config = WasiConfig::new()
            .with_seed([42; 32])
            .with_env_var("TEST", "value")
            .with_memory_range(100..110);

        assert_eq!(config.seed, Some([42; 32]));
        assert_eq!(
            config.env_vars,
            vec![("TEST".to_string(), "value".to_string())]
        );
        assert_eq!(config.memory_range, 100..110);
    }

    #[test]
    fn test_wasi_extension_requirements() {
        let req = WasiExtension::init_requirements();
        assert_eq!(req.memory_range, Some(200..210));
        assert!(req.requires_random_seed);
    }
}
