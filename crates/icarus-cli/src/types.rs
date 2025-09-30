//! Domain-specific newtypes for type safety in icarus-cli.
//!
//! Following the newtype pattern from `rust_best_practices.md` Section 3,
//! these types provide compile-time safety and single validation points.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A validated MCP server name.
///
/// Ensures server names are non-empty and follow naming conventions.
///
/// # Examples
///
/// ```
/// # use icarus_cli::ServerName;
/// let name = ServerName::new("my-mcp-server")?;
/// assert_eq!(name.as_str(), "my-mcp-server");
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServerName(String);

impl ServerName {
    /// Creates a new validated `ServerName`.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is empty or contains invalid characters.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();

        if name.is_empty() {
            return Err(anyhow!("Server name cannot be empty"));
        }

        if name.len() > 255 {
            return Err(anyhow!("Server name cannot exceed 255 characters"));
        }

        // Allow alphanumeric, hyphens, underscores
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow!(
                "Server name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }

        Ok(Self(name))
    }

    /// Returns the server name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ServerName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for ServerName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ServerName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for ServerName {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

/// Internet Computer canister ID with validation.
///
/// Validates canister ID format (contains hyphens and meets length requirements).
///
/// # Examples
///
/// ```
/// # use icarus_cli::CanisterId;
/// let id = CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
/// assert!(id.as_str().contains('-'));
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CanisterId(String);

impl CanisterId {
    /// Creates a new validated `CanisterId`.
    ///
    /// # Errors
    ///
    /// Returns an error if the canister ID format is invalid.
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();

        if id.is_empty() {
            return Err(anyhow!("Canister ID cannot be empty"));
        }

        // Basic validation: must contain hyphens and be at least 20 characters
        // (IC canister IDs are typically 27 characters with 4 hyphens)
        if !id.contains('-') || id.len() < 20 {
            return Err(anyhow!("Invalid canister ID format: {}", id));
        }

        Ok(Self(id))
    }

    /// Returns the canister ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CanisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for CanisterId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for CanisterId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for CanisterId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for CanisterId {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

/// Internet Computer network identifier.
///
/// Represents the different IC networks available for deployment.
///
/// # Examples
///
/// ```
/// # use icarus_cli::Network;
/// let network = Network::Local;
/// assert_eq!(network.as_str(), "local");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    /// Local dfx replica
    Local,
    /// Internet Computer mainnet
    Ic,
    /// Internet Computer testnet
    Testnet,
}

impl Network {
    /// Returns the network as a string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Ic => "ic",
            Self::Testnet => "testnet",
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "ic" | "mainnet" => Ok(Self::Ic),
            "testnet" => Ok(Self::Testnet),
            _ => Err(anyhow!("Unknown network: {}", s)),
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::Local
    }
}

impl PartialEq<str> for Network {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Network {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_name_validation() {
        // Valid names
        assert!(ServerName::new("valid-name").is_ok());
        assert!(ServerName::new("valid_name").is_ok());
        assert!(ServerName::new("validname123").is_ok());
        assert!(ServerName::new("a").is_ok());

        // Invalid names
        assert!(ServerName::new("").is_err());
        assert!(ServerName::new("invalid@name").is_err());
        assert!(ServerName::new("invalid name").is_err());
        assert!(ServerName::new("invalid.name").is_err());

        // Too long
        assert!(ServerName::new("a".repeat(256)).is_err());
    }

    #[test]
    fn test_canister_id_validation() {
        // Valid IDs
        assert!(CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").is_ok());
        assert!(CanisterId::new("rrkah-fqaaa-aaaaa-aaaaq-cai").is_ok());

        // Invalid IDs
        assert!(CanisterId::new("").is_err());
        assert!(CanisterId::new("invalid").is_err());
        assert!(CanisterId::new("too-short").is_err());
        assert!(CanisterId::new("nohyphens").is_err());
    }

    #[test]
    fn test_network_parsing() {
        assert_eq!("local".parse::<Network>().unwrap(), Network::Local);
        assert_eq!("LOCAL".parse::<Network>().unwrap(), Network::Local);
        assert_eq!("ic".parse::<Network>().unwrap(), Network::Ic);
        assert_eq!("mainnet".parse::<Network>().unwrap(), Network::Ic);
        assert_eq!("testnet".parse::<Network>().unwrap(), Network::Testnet);

        assert!("invalid".parse::<Network>().is_err());
    }

    #[test]
    fn test_network_display() {
        assert_eq!(Network::Local.to_string(), "local");
        assert_eq!(Network::Ic.to_string(), "ic");
        assert_eq!(Network::Testnet.to_string(), "testnet");
    }

    #[test]
    fn test_server_name_as_ref() {
        let name = ServerName::new("test").unwrap();
        let s: &str = name.as_ref();
        assert_eq!(s, "test");
    }

    #[test]
    fn test_canister_id_as_ref() {
        let id = CanisterId::new("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
        let s: &str = id.as_ref();
        assert_eq!(s, "rdmx6-jaaaa-aaaaa-aaadq-cai");
    }
}
