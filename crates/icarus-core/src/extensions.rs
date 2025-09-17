//! Extension system for composable canister initialization
//!
//! This module provides a trait-based system for extending canister initialization
//! beyond the basic authentication setup. Extensions can be composed together to
//! provide additional capabilities like WASI support, custom memory management,
//! or other system-level integrations.

use std::fmt;

/// Error type for initialization failures
#[derive(Debug, Clone)]
pub enum InitError {
    /// Extension failed to initialize with a specific message
    InitializationFailed(String),
    /// Required dependency was not available
    DependencyMissing(String),
    /// Configuration was invalid
    InvalidConfiguration(String),
    /// System resource was unavailable
    ResourceUnavailable(String),
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            InitError::DependencyMissing(dep) => write!(f, "Missing dependency: {}", dep),
            InitError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            InitError::ResourceUnavailable(res) => write!(f, "Resource unavailable: {}", res),
        }
    }
}

impl std::error::Error for InitError {}

/// Requirements that an extension may have for proper initialization
#[derive(Debug, Clone, Default)]
pub struct InitRequirements {
    /// Memory range required for stable memory operations
    pub memory_range: Option<std::ops::Range<u8>>,
    /// Whether this extension requires a random seed
    pub requires_random_seed: bool,
    /// Environment variables required by the extension
    pub required_env_vars: Vec<&'static str>,
    /// Other extensions this one depends on
    pub dependencies: Vec<&'static str>,
}

impl InitRequirements {
    /// Create empty requirements
    pub const fn new() -> Self {
        Self {
            memory_range: None,
            requires_random_seed: false,
            required_env_vars: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set memory range requirement
    pub fn with_memory_range(mut self, range: std::ops::Range<u8>) -> Self {
        self.memory_range = Some(range);
        self
    }

    /// Set random seed requirement
    pub const fn with_random_seed(mut self) -> Self {
        self.requires_random_seed = true;
        self
    }

    /// Add environment variable requirement
    pub fn with_env_var(mut self, var: &'static str) -> Self {
        self.required_env_vars.push(var);
        self
    }

    /// Add dependency on another extension
    pub fn with_dependency(mut self, dep: &'static str) -> Self {
        self.dependencies.push(dep);
        self
    }
}

/// Trait for composable initialization extensions
pub trait InitializationExtension {
    /// Configuration type for this extension
    type Config: Default;

    /// Name of this extension (for dependency resolution and debugging)
    const NAME: &'static str;

    /// Return the initialization requirements for this extension
    fn init_requirements() -> InitRequirements {
        InitRequirements::new()
    }

    /// Initialize this extension with the given configuration
    ///
    /// This method is called during canister initialization, after authentication
    /// has been set up but before any tool methods can be called.
    fn initialize(config: Self::Config) -> Result<Self, InitError>
    where
        Self: Sized;

    /// Optional cleanup method called during pre_upgrade
    fn pre_upgrade(&self) -> Result<(), InitError> {
        Ok(())
    }

    /// Optional restoration method called during post_upgrade
    fn post_upgrade(&mut self) -> Result<(), InitError> {
        Ok(())
    }
}

/// Marker trait for services that provide extensions
pub trait ExtensionProvider {
    /// Initialize all extensions for this service
    fn init_extensions() -> Result<(), InitError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Handle pre-upgrade for all extensions
    fn pre_upgrade_extensions() -> Result<(), InitError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Handle post-upgrade for all extensions
    fn post_upgrade_extensions() -> Result<(), InitError> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Default implementation for services without extensions
impl<T> ExtensionProvider for T {
    fn init_extensions() -> Result<(), InitError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtension {
        initialized: bool,
    }

    impl InitializationExtension for TestExtension {
        type Config = ();
        const NAME: &'static str = "test";

        fn init_requirements() -> InitRequirements {
            InitRequirements::new()
                .with_random_seed()
                .with_env_var("TEST_VAR")
        }

        fn initialize(_config: Self::Config) -> Result<Self, InitError> {
            Ok(Self { initialized: true })
        }
    }

    #[test]
    fn test_extension_requirements() {
        let req = TestExtension::init_requirements();
        assert!(req.requires_random_seed);
        assert_eq!(req.required_env_vars, vec!["TEST_VAR"]);
    }

    #[test]
    fn test_extension_initialization() {
        let ext = TestExtension::initialize(()).unwrap();
        assert!(ext.initialized);
    }

    #[test]
    fn test_error_display() {
        let error = InitError::InitializationFailed("test error".to_string());
        assert_eq!(error.to_string(), "Initialization failed: test error");
    }
}
