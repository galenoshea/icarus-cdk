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

#[cfg(test)]
mod tests {
    use super::*;

    // Test structures for Storable trait
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestUser {
        id: u64,
        name: String,
        active: bool,
    }

    impl Storable for TestUser {
        fn to_bytes(&self) -> Result<Vec<u8>> {
            Ok(serde_json::to_vec(self)?)
        }

        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(serde_json::from_slice(bytes)?)
        }

        const MAX_SIZE: u32 = 1024;
        const FIXED_SIZE: Option<u32> = None;
    }

    // Test structure with fixed size
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestCounter {
        count: u64,
    }

    impl Storable for TestCounter {
        fn to_bytes(&self) -> Result<Vec<u8>> {
            Ok(self.count.to_le_bytes().to_vec())
        }

        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != 8 {
                return Err(crate::error::IcarusError::State(
                    "Invalid data length for TestCounter".to_string(),
                ));
            }
            let count = u64::from_le_bytes(bytes.try_into().unwrap());
            Ok(TestCounter { count })
        }

        const MAX_SIZE: u32 = 8;
        const FIXED_SIZE: Option<u32> = Some(8);
    }

    // Test structure for Migratable trait
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        version: u32,
        setting1: String,
        setting2: Option<bool>, // Added in version 2
    }

    impl Storable for TestConfig {
        fn to_bytes(&self) -> Result<Vec<u8>> {
            Ok(serde_json::to_vec(self)?)
        }

        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(serde_json::from_slice(bytes)?)
        }

        const MAX_SIZE: u32 = 512;
        const FIXED_SIZE: Option<u32> = None;
    }

    impl Migratable for TestConfig {
        fn current_version() -> u32 {
            2
        }

        fn migrate_from(version: u32, data: &[u8]) -> Result<Self> {
            match version {
                1 => {
                    // Version 1 format: {"version": 1, "setting1": "value"}
                    #[derive(Deserialize)]
                    struct V1Config {
                        version: u32,
                        setting1: String,
                    }

                    let v1: V1Config = serde_json::from_slice(data)?;

                    Ok(TestConfig {
                        version: 2,
                        setting1: v1.setting1,
                        setting2: Some(true), // Default value for new field
                    })
                }
                2 => {
                    // Current version, no migration needed
                    Self::from_bytes(data)
                }
                _ => Err(crate::error::IcarusError::State(format!(
                    "Unsupported version: {}",
                    version
                ))),
            }
        }
    }

    #[test]
    fn test_storable_user_serialization() {
        let user = TestUser {
            id: 123,
            name: "Alice".to_string(),
            active: true,
        };

        let bytes = user.to_bytes().unwrap();
        let restored = TestUser::from_bytes(&bytes).unwrap();

        assert_eq!(restored, user);
    }

    #[test]
    fn test_storable_user_constants() {
        assert_eq!(TestUser::MAX_SIZE, 1024);
        assert_eq!(TestUser::FIXED_SIZE, None);
    }

    #[test]
    fn test_storable_counter_serialization() {
        let counter = TestCounter { count: 42 };

        let bytes = counter.to_bytes().unwrap();
        assert_eq!(bytes.len(), 8); // u64 is 8 bytes

        let restored = TestCounter::from_bytes(&bytes).unwrap();
        assert_eq!(restored, counter);
    }

    #[test]
    fn test_storable_counter_constants() {
        assert_eq!(TestCounter::MAX_SIZE, 8);
        assert_eq!(TestCounter::FIXED_SIZE, Some(8));
    }

    #[test]
    fn test_storable_counter_invalid_length() {
        let invalid_bytes = vec![1, 2, 3]; // Wrong length
        let result = TestCounter::from_bytes(&invalid_bytes);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Invalid data length"));
    }

    #[test]
    fn test_storable_user_large_name() {
        let user = TestUser {
            id: 456,
            name: "A".repeat(500), // Large name
            active: false,
        };

        let bytes = user.to_bytes().unwrap();
        let restored = TestUser::from_bytes(&bytes).unwrap();

        assert_eq!(restored.name.len(), 500);
        assert_eq!(restored, user);
    }

    #[test]
    fn test_storable_user_special_characters() {
        let user = TestUser {
            id: 789,
            name: "Test 🚀 User with émojis & spëcial chars!".to_string(),
            active: true,
        };

        let bytes = user.to_bytes().unwrap();
        let restored = TestUser::from_bytes(&bytes).unwrap();

        assert_eq!(restored, user);
    }

    #[test]
    fn test_storable_user_invalid_json() {
        let invalid_bytes = b"invalid json {";
        let result = TestUser::from_bytes(invalid_bytes);
        assert!(result.is_err());
        assert!(result.is_err());
    }

    #[test]
    fn test_state_version_creation() {
        let version = StateVersion {
            version: 1,
            created_at: 1234567890,
        };

        assert_eq!(version.version, 1);
        assert_eq!(version.created_at, 1234567890);
    }

    #[test]
    fn test_state_version_clone() {
        let version1 = StateVersion {
            version: 2,
            created_at: 9876543210,
        };

        let version2 = version1.clone();
        assert_eq!(version2.version, 2);
        assert_eq!(version2.created_at, 9876543210);
    }

    #[test]
    fn test_state_version_debug() {
        let version = StateVersion {
            version: 3,
            created_at: 1111111111,
        };

        let debug_str = format!("{:?}", version);
        assert!(debug_str.contains("version: 3"));
        assert!(debug_str.contains("created_at: 1111111111"));
    }

    #[test]
    fn test_state_version_serialization() {
        let version = StateVersion {
            version: 5,
            created_at: 2222222222,
        };

        let json = serde_json::to_string(&version).unwrap();
        let restored: StateVersion = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.version, version.version);
        assert_eq!(restored.created_at, version.created_at);
    }

    #[test]
    fn test_migratable_current_version() {
        assert_eq!(TestConfig::current_version(), 2);
    }

    #[test]
    fn test_migratable_v2_no_migration() {
        let config = TestConfig {
            version: 2,
            setting1: "test".to_string(),
            setting2: Some(false),
        };

        let bytes = config.to_bytes().unwrap();
        let migrated = TestConfig::migrate_from(2, &bytes).unwrap();

        assert_eq!(migrated, config);
    }

    #[test]
    fn test_migratable_v1_to_v2_migration() {
        // Create v1 format data
        let v1_json = r#"{"version": 1, "setting1": "legacy_value"}"#;
        let v1_bytes = v1_json.as_bytes();

        let migrated = TestConfig::migrate_from(1, v1_bytes).unwrap();

        assert_eq!(migrated.version, 2);
        assert_eq!(migrated.setting1, "legacy_value");
        assert_eq!(migrated.setting2, Some(true)); // Default value
    }

    #[test]
    fn test_migratable_unsupported_version() {
        let data = b"some data";
        let result = TestConfig::migrate_from(999, data);

        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Unsupported version: 999"));
    }

    #[test]
    fn test_migratable_invalid_v1_data() {
        let invalid_v1 = b"invalid v1 json";
        let result = TestConfig::migrate_from(1, invalid_v1);

        assert!(result.is_err());
        assert!(result.is_err());
    }

    #[test]
    fn test_state_metadata_creation() {
        let version = StateVersion {
            version: 1,
            created_at: 1000000000,
        };

        let metadata = StateMetadata {
            version,
            size_bytes: 2048,
            last_modified: 2000000000,
        };

        assert_eq!(metadata.version.version, 1);
        assert_eq!(metadata.size_bytes, 2048);
        assert_eq!(metadata.last_modified, 2000000000);
    }

    #[test]
    fn test_state_metadata_clone() {
        let metadata1 = StateMetadata {
            version: StateVersion {
                version: 3,
                created_at: 1500000000,
            },
            size_bytes: 4096,
            last_modified: 3000000000,
        };

        let metadata2 = metadata1.clone();
        assert_eq!(metadata2.version.version, 3);
        assert_eq!(metadata2.size_bytes, 4096);
        assert_eq!(metadata2.last_modified, 3000000000);
    }

    #[test]
    fn test_state_metadata_debug() {
        let metadata = StateMetadata {
            version: StateVersion {
                version: 2,
                created_at: 1111111111,
            },
            size_bytes: 1024,
            last_modified: 2222222222,
        };

        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("version: 2"));
        assert!(debug_str.contains("size_bytes: 1024"));
        assert!(debug_str.contains("last_modified: 2222222222"));
    }

    #[test]
    fn test_state_metadata_serialization() {
        let metadata = StateMetadata {
            version: StateVersion {
                version: 4,
                created_at: 1234567890,
            },
            size_bytes: 8192,
            last_modified: 9876543210,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let restored: StateMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.version.version, metadata.version.version);
        assert_eq!(restored.size_bytes, metadata.size_bytes);
        assert_eq!(restored.last_modified, metadata.last_modified);
    }

    #[test]
    fn test_counter_extreme_values() {
        let counter = TestCounter { count: u64::MAX };

        let bytes = counter.to_bytes().unwrap();
        let restored = TestCounter::from_bytes(&bytes).unwrap();

        assert_eq!(restored.count, u64::MAX);
    }

    #[test]
    fn test_counter_zero_value() {
        let counter = TestCounter { count: 0 };

        let bytes = counter.to_bytes().unwrap();
        let restored = TestCounter::from_bytes(&bytes).unwrap();

        assert_eq!(restored.count, 0);
    }

    #[test]
    fn test_user_empty_name() {
        let user = TestUser {
            id: 0,
            name: "".to_string(),
            active: false,
        };

        let bytes = user.to_bytes().unwrap();
        let restored = TestUser::from_bytes(&bytes).unwrap();

        assert_eq!(restored, user);
        assert!(restored.name.is_empty());
    }

    #[test]
    fn test_migration_chain() {
        // Test that we can migrate from v1 to current (v2)
        let v1_json = r#"{"version": 1, "setting1": "original"}"#;
        let migrated = TestConfig::migrate_from(1, v1_json.as_bytes()).unwrap();

        // Now serialize the migrated config
        let v2_bytes = migrated.to_bytes().unwrap();

        // And "migrate" it again (should be no-op)
        let final_config = TestConfig::migrate_from(2, &v2_bytes).unwrap();

        assert_eq!(final_config.version, 2);
        assert_eq!(final_config.setting1, "original");
        assert_eq!(final_config.setting2, Some(true));
    }

    #[test]
    fn test_storable_trait_methods() {
        // Test that Storable trait methods work correctly
        let user = TestUser {
            id: 1,
            name: "test".to_string(),
            active: true,
        };

        // Test trait methods work
        let _bytes = user.to_bytes().unwrap();
        assert_eq!(TestUser::MAX_SIZE, 1024);
        assert_eq!(TestUser::FIXED_SIZE, None);
    }
}
