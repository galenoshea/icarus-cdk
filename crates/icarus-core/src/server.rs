//! Server trait and implementation for Icarus MCP servers

use crate::error::Result;
use crate::tool::IcarusTool;
use async_trait::async_trait;
use candid::Principal;
use rmcp::ServerHandler;
use std::fmt;

/// Version information for server upgrades
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Major version number
    pub major: u8,
    /// Minor version number
    pub minor: u8,
    /// Patch version number
    pub patch: u8,
}

impl Version {
    /// Create a new version
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string (e.g., "1.2.3")
    pub fn parse(version_str: &str) -> std::result::Result<Self, String> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format 'major.minor.patch'".to_string());
        }

        let major = parts[0].parse::<u8>()
            .map_err(|_| "Major version must be a valid u8".to_string())?;
        let minor = parts[1].parse::<u8>()
            .map_err(|_| "Minor version must be a valid u8".to_string())?;
        let patch = parts[2].parse::<u8>()
            .map_err(|_| "Patch version must be a valid u8".to_string())?;

        Ok(Self::new(major, minor, patch))
    }

    /// Check if this version is compatible with another version
    /// Compatible means same major version, and this version >= other
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        if self.major != other.major {
            return false;
        }
        self >= other
    }

    /// Check if this is a breaking change from another version
    pub fn is_breaking_change_from(&self, other: &Version) -> bool {
        self.major > other.major
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                std::cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

/// Core trait for Icarus MCP servers that extends rmcp's ServerHandler
#[async_trait]
pub trait IcarusServer: ServerHandler + Send + Sync {
    /// Called when the canister is first initialized
    async fn on_canister_init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called during canister upgrades
    async fn on_canister_upgrade(&mut self, _from_version: Version) -> Result<()> {
        Ok(())
    }

    /// Called before canister upgrade to save state
    async fn on_pre_upgrade(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called after canister upgrade to restore state
    async fn on_post_upgrade(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the canister principal
    fn canister_id(&self) -> Option<Principal> {
        None
    }

    /// Register a tool with the server
    fn register_tool(&mut self, tool: Box<dyn IcarusTool>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_new() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_display() {
        let version = Version::new(2, 5, 10);
        assert_eq!(version.to_string(), "2.5.10");
    }

    #[test]
    fn test_version_parse_valid() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_invalid_format() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("").is_err());
    }

    #[test]
    fn test_version_parse_invalid_numbers() {
        assert!(Version::parse("a.2.3").is_err());
        assert!(Version::parse("1.b.3").is_err());
        assert!(Version::parse("1.2.c").is_err());
        assert!(Version::parse("256.2.3").is_err()); // u8 overflow
    }

    #[test]
    fn test_version_equality() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 3);
        let v3 = Version::new(1, 2, 4);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_version_ordering() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v1_1_1 = Version::new(1, 1, 1);
        let v2_0_0 = Version::new(2, 0, 0);

        // Major version ordering
        assert!(v2_0_0 > v1_1_1);
        assert!(v1_0_0 < v2_0_0);

        // Minor version ordering (same major)
        assert!(v1_1_0 > v1_0_0);
        assert!(v1_0_0 < v1_1_0);

        // Patch version ordering (same major.minor)
        assert!(v1_1_1 > v1_1_0);
        assert!(v1_1_0 < v1_1_1);
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v1_2_5 = Version::new(1, 2, 5);
        let v2_0_0 = Version::new(2, 0, 0);

        // Same major version, newer minor/patch is compatible
        assert!(v1_2_5.is_compatible_with(&v1_0_0));
        assert!(v1_2_5.is_compatible_with(&v1_1_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));

        // Same version is compatible
        assert!(v1_1_0.is_compatible_with(&v1_1_0));

        // Older version is not compatible with newer
        assert!(!v1_0_0.is_compatible_with(&v1_1_0));
        assert!(!v1_1_0.is_compatible_with(&v1_2_5));

        // Different major version is not compatible
        assert!(!v2_0_0.is_compatible_with(&v1_2_5));
        assert!(!v1_2_5.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_version_breaking_changes() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v1_2_5 = Version::new(1, 2, 5);
        let v2_0_0 = Version::new(2, 0, 0);
        let v3_0_0 = Version::new(3, 0, 0);

        // Major version changes are breaking
        assert!(v2_0_0.is_breaking_change_from(&v1_2_5));
        assert!(v3_0_0.is_breaking_change_from(&v2_0_0));
        assert!(v3_0_0.is_breaking_change_from(&v1_0_0));

        // Minor and patch changes are not breaking
        assert!(!v1_1_0.is_breaking_change_from(&v1_0_0));
        assert!(!v1_2_5.is_breaking_change_from(&v1_1_0));

        // Same version is not breaking
        assert!(!v1_1_0.is_breaking_change_from(&v1_1_0));

        // Downgrade is not considered breaking
        assert!(!v1_0_0.is_breaking_change_from(&v1_1_0));
    }

    #[test]
    fn test_version_clone() {
        let v1 = Version::new(1, 2, 3);
        let v2 = v1.clone();
        assert_eq!(v1, v2);
        assert_eq!(v1.major, v2.major);
        assert_eq!(v1.minor, v2.minor);
        assert_eq!(v1.patch, v2.patch);
    }

    #[test]
    fn test_version_debug() {
        let version = Version::new(1, 2, 3);
        let debug_str = format!("{:?}", version);
        assert!(debug_str.contains("major: 1"));
        assert!(debug_str.contains("minor: 2"));
        assert!(debug_str.contains("patch: 3"));
    }

    #[test]
    fn test_version_edge_cases() {
        // Test maximum values
        let max_version = Version::new(255, 255, 255);
        assert_eq!(max_version.to_string(), "255.255.255");

        // Test minimum values
        let min_version = Version::new(0, 0, 0);
        assert_eq!(min_version.to_string(), "0.0.0");

        // Test ordering with max values
        assert!(max_version > min_version);

        // Different major versions are not compatible
        assert!(!max_version.is_compatible_with(&min_version));

        // Same major version should be compatible
        let v0_0_1 = Version::new(0, 0, 1);
        assert!(v0_0_1.is_compatible_with(&min_version));
    }

    #[test]
    fn test_version_parse_edge_cases() {
        // Test valid edge cases
        assert_eq!(Version::parse("0.0.0").unwrap(), Version::new(0, 0, 0));
        assert_eq!(Version::parse("255.255.255").unwrap(), Version::new(255, 255, 255));

        // Test parsing with spaces (should fail)
        assert!(Version::parse(" 1.2.3").is_err());
        assert!(Version::parse("1.2.3 ").is_err());
        assert!(Version::parse("1. 2.3").is_err());
    }
}
