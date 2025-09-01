//! State management abstractions for persistent storage

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Trait for types that can be stored in ICP stable memory
pub trait Storable: Sized {
    /// Serialize the type to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    /// Maximum size in bytes when serialized
    const MAX_SIZE: u32;

    /// Fixed size in bytes (None if variable size)
    const FIXED_SIZE: Option<u32> = None;
}

/// Version information for state migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    pub version: u32,
    pub created_at: u64,
}

/// Trait for state that can be migrated between versions
pub trait Migratable: Storable {
    /// Current version of the state schema
    fn current_version() -> u32;

    /// Migrate from an older version
    fn migrate_from(version: u32, data: &[u8]) -> Result<Self>;
}

/// Metadata about server state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub version: StateVersion,
    pub size_bytes: u64,
    pub last_modified: u64,
}
