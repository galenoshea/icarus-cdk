//! Domain newtypes for type safety following `rust_best_practices.md` patterns.
//!
//! This module provides type-safe wrappers for primitive types used throughout
//! the Icarus CDK. All newtypes include validation and implement the standard
//! traits (Debug, Clone, `PartialEq`, etc.) consistently.

use std::fmt;
use std::str::FromStr;

use candid::{CandidType, Deserialize};
#[cfg(feature = "ic-canister")]
use ic_cdk::api::time;
use serde::Serialize;
#[cfg(not(feature = "ic-canister"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::IcarusError;

/// Type-safe tool identifier with validation.
///
/// Tool IDs must be valid identifiers that can be used in MCP protocol.
/// They follow the pattern: `[namespace.]tool_name` where namespace is optional.
///
/// # Examples
///
/// ```rust
/// use icarus_core::ToolId;
///
/// // Simple tool name
/// let tool_id = ToolId::new("add")?;
///
/// // Namespaced tool name
/// let tool_id = ToolId::new("calculator.add")?;
///
/// // Invalid names will fail
/// assert!(ToolId::new("").is_err());
/// assert!(ToolId::new("invalid name").is_err());
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize, Serialize)]
#[repr(transparent)]
pub struct ToolId(String);

impl ToolId {
    /// Creates a new tool ID with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidToolId` if the ID is invalid:
    /// - Empty string
    /// - Contains whitespace
    /// - Contains invalid characters
    /// - Exceeds maximum length
    pub fn new(id: impl Into<String>) -> Result<Self, IcarusError> {
        let id = id.into();

        if id.is_empty() {
            return Err(IcarusError::InvalidToolId(
                "Tool ID cannot be empty".to_string(),
            ));
        }

        if id.len() > crate::MAX_TOOL_NAME_LENGTH {
            return Err(IcarusError::InvalidToolId(format!(
                "Tool ID exceeds maximum length of {}",
                crate::MAX_TOOL_NAME_LENGTH
            )));
        }

        if id.contains(char::is_whitespace) {
            return Err(IcarusError::InvalidToolId(
                "Tool ID cannot contain whitespace".to_string(),
            ));
        }

        // Tool IDs must start with a letter and contain only ASCII identifiers
        let mut chars = id.chars();
        if let Some(first_char) = chars.next() {
            if !first_char.is_ascii_alphabetic() {
                return Err(IcarusError::InvalidToolId(
                    "Tool ID must start with a letter".to_string(),
                ));
            }
        }

        // Rest of the characters must be alphanumeric, underscore, dot, or hyphen
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
        {
            return Err(IcarusError::InvalidToolId(
                "Tool ID contains invalid characters".to_string(),
            ));
        }

        Ok(Self(id))
    }

    /// Returns the tool ID as a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `ToolId` and returns the inner String.
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for ToolId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ToolId {
    type Err = IcarusError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl From<ToolId> for String {
    #[inline]
    fn from(tool_id: ToolId) -> Self {
        tool_id.0
    }
}

impl AsRef<str> for ToolId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Type-safe user identifier with validation.
///
/// User IDs are typically Internet Computer Principal IDs or other
/// authentication system identifiers.
///
/// # Examples
///
/// ```rust
/// use icarus_core::UserId;
///
/// let user_id = UserId::new("rdmx6-jaaaa-aaaah-qcaiq-cai")?;
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize, Serialize)]
#[repr(transparent)]
pub struct UserId(String);

impl UserId {
    /// Creates a new user ID with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidUserId` if the ID is invalid.
    pub fn new(id: impl Into<String>) -> Result<Self, IcarusError> {
        let id = id.into();

        if id.is_empty() {
            return Err(IcarusError::InvalidUserId(
                "User ID cannot be empty".to_string(),
            ));
        }

        if id.len() > 255 {
            return Err(IcarusError::InvalidUserId("User ID too long".to_string()));
        }

        Ok(Self(id))
    }

    /// Returns the user ID as a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `UserId` and returns the inner String.
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for UserId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = IcarusError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for UserId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Type-safe session identifier for tracking user sessions.
///
/// Session IDs are used to track tool execution contexts and maintain
/// state across multiple tool calls.
///
/// # Examples
///
/// ```rust
/// use icarus_core::SessionId;
///
/// // Generate a new session ID
/// let session_id = SessionId::generate();
///
/// // Create from existing string
/// let session_id = SessionId::new("session_12345")?;
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize, Serialize)]
#[repr(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a new session ID with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidSessionId` if the ID is invalid.
    pub fn new(id: impl Into<String>) -> Result<Self, IcarusError> {
        let id = id.into();

        if id.is_empty() {
            return Err(IcarusError::InvalidSessionId(
                "Session ID cannot be empty".to_string(),
            ));
        }

        if id.len() > 128 {
            return Err(IcarusError::InvalidSessionId(
                "Session ID too long".to_string(),
            ));
        }

        Ok(Self(id))
    }

    /// Generates a new unique session ID.
    ///
    /// Uses current timestamp and a deterministic hash to ensure uniqueness.
    /// This follows `rust_best_practices.md` by avoiding getrandom dependency.
    ///
    /// # Panics
    ///
    /// May panic if system time is before Unix epoch (extremely unlikely).
    #[must_use]
    #[inline]
    pub fn generate() -> Self {
        #[cfg(not(feature = "ic-canister"))]
        let timestamp = {
            use std::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            #[allow(clippy::cast_possible_truncation)]
            let base_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time before Unix epoch")
                .as_nanos() as u64;
            // Add atomic counter to ensure uniqueness in tests
            base_time + COUNTER.fetch_add(1, Ordering::SeqCst)
        };
        #[cfg(feature = "ic-canister")]
        let timestamp = time();

        // Use a simple hash of timestamp for pseudo-randomness
        // This is deterministic but provides sufficient uniqueness for session IDs
        let hash = {
            let mut hasher = timestamp;
            hasher ^= hasher >> 16;
            hasher = hasher.wrapping_mul(0x045d_9f3b);
            hasher ^= hasher >> 16;
            hasher = hasher.wrapping_mul(0x045d_9f3b);
            hasher ^= hasher >> 16;
            hasher
        };

        let session_id = format!("sess_{timestamp:x}_{hash:x}");

        // Safe to unwrap since we control the format
        Self(session_id)
    }

    /// Returns the session ID as a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `SessionId` and returns the inner String.
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for SessionId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = IcarusError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for SessionId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Type-safe timestamp wrapper for Internet Computer time values.
///
/// Timestamps are nanoseconds since Unix epoch, as used by the IC.
///
/// # Examples
///
/// ```rust
/// use icarus_core::Timestamp;
///
/// // Current time
/// let now = Timestamp::now();
///
/// // From nanoseconds
/// let timestamp = Timestamp::from_nanos(1_000_000_000_000);
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, CandidType, Deserialize, Serialize,
)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Creates a timestamp from nanoseconds since Unix epoch.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Returns the current timestamp.
    ///
    /// # Panics
    ///
    /// May panic if system time is before Unix epoch (extremely unlikely).
    #[must_use]
    #[inline]
    pub fn now() -> Self {
        #[cfg(not(feature = "ic-canister"))]
        {
            #[allow(clippy::cast_possible_truncation)]
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time before Unix epoch")
                .as_nanos() as u64;
            Self(nanos)
        }
        #[cfg(feature = "ic-canister")]
        {
            Self(time())
        }
    }

    /// Returns the timestamp as nanoseconds since Unix epoch.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Returns the timestamp as seconds since Unix epoch.
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Returns the timestamp as milliseconds since Unix epoch.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0 / 1_000_000
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as ISO 8601 timestamp for readability
        #[allow(clippy::cast_possible_wrap)]
        let secs = self.as_secs() as i64;
        let nanos = (self.0 % 1_000_000_000) as u32;

        match chrono::DateTime::from_timestamp(secs, nanos) {
            Some(dt) => write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ")),
            None => write!(f, "{}ns", self.0),
        }
    }
}

impl From<u64> for Timestamp {
    #[inline]
    fn from(nanos: u64) -> Self {
        Self::from_nanos(nanos)
    }
}

impl From<Timestamp> for u64 {
    #[inline]
    fn from(timestamp: Timestamp) -> Self {
        timestamp.as_nanos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_id_validation() {
        // Valid tool IDs
        assert!(ToolId::new("add").is_ok());
        assert!(ToolId::new("calculator.add").is_ok());
        assert!(ToolId::new("my-tool_v2").is_ok());

        // Invalid tool IDs
        assert!(ToolId::new("").is_err());
        assert!(ToolId::new("invalid name").is_err());
        assert!(ToolId::new("invalid@name").is_err());
        assert!(ToolId::new("a".repeat(300)).is_err());
    }

    #[test]
    fn test_tool_id_as_str() {
        let tool_id = ToolId::new("test_tool").expect("test value should be valid");
        assert_eq!(tool_id.as_str(), "test_tool");
    }

    #[test]
    fn test_tool_id_into_string() {
        let tool_id = ToolId::new("test_tool").expect("test value should be valid");
        assert_eq!(tool_id.into_string(), "test_tool");
    }

    #[test]
    fn test_tool_id_display() {
        let tool_id = ToolId::new("calculator.add").expect("test value should be valid");
        assert_eq!(format!("{tool_id}"), "calculator.add");
    }

    #[test]
    fn test_tool_id_from_str() {
        let tool_id: ToolId = "valid_tool".parse().expect("test value should be valid");
        assert_eq!(tool_id.as_str(), "valid_tool");

        let result: Result<ToolId, _> = "invalid name".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_id_namespaced() {
        let tool_id = ToolId::new("namespace.tool.action").expect("test value should be valid");
        assert_eq!(tool_id.as_str(), "namespace.tool.action");
    }

    #[test]
    fn test_tool_id_special_chars() {
        // Valid special characters
        assert!(ToolId::new("tool_123").is_ok());
        assert!(ToolId::new("tool-v2").is_ok());
        assert!(ToolId::new("ns.tool").is_ok());

        // Invalid special characters
        assert!(ToolId::new("tool@domain").is_err());
        assert!(ToolId::new("tool:action").is_err());
        assert!(ToolId::new("tool/action").is_err());
        assert!(ToolId::new("tool space").is_err());
    }

    #[test]
    fn test_user_id_validation() {
        assert!(UserId::new("user123").is_ok());
        assert!(UserId::new("rdmx6-jaaaa-aaaah-qcaiq-cai").is_ok());

        assert!(UserId::new("").is_err());
        assert!(UserId::new("a".repeat(300)).is_err());
    }

    #[test]
    fn test_user_id_operations() {
        let user_id = UserId::new("test_user").expect("test value should be valid");
        assert_eq!(user_id.as_str(), "test_user");
        assert_eq!(user_id.to_string(), "test_user");

        let user_id2 = user_id.clone();
        assert_eq!(user_id, user_id2);

        assert_eq!(user_id.into_string(), "test_user");
    }

    #[test]
    fn test_user_id_from_str() {
        let user_id: UserId = "valid_user".parse().expect("test value should be valid");
        assert_eq!(user_id.as_str(), "valid_user");

        let result: Result<UserId, _> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_session_id_validation() {
        assert!(SessionId::new("session123").is_ok());
        assert!(SessionId::new("sess_abc123").is_ok());

        assert!(SessionId::new("").is_err());
        assert!(SessionId::new("a".repeat(200)).is_err());
    }

    #[test]
    fn test_session_id_generation() {
        let session1 = SessionId::generate();
        let session2 = SessionId::generate();

        // Sessions should be different
        assert_ne!(session1, session2);

        // Should start with prefix
        assert!(session1.as_str().starts_with("sess_"));
        assert!(session2.as_str().starts_with("sess_"));

        // Should contain timestamp and hash components
        assert!(session1.as_str().contains('_'));
        let parts: Vec<&str> = session1.as_str().split('_').collect();
        assert_eq!(parts.len(), 3); // "sess", timestamp, hash
    }

    #[test]
    fn test_session_id_operations() {
        let session_id = SessionId::new("test_session").expect("test value should be valid");
        assert_eq!(session_id.as_str(), "test_session");
        assert_eq!(session_id.to_string(), "test_session");

        let session_id2 = session_id.clone();
        assert_eq!(session_id, session_id2);

        assert_eq!(session_id.into_string(), "test_session");
    }

    #[test]
    fn test_session_id_from_str() {
        let session_id: SessionId = "valid_session".parse().expect("test value should be valid");
        assert_eq!(session_id.as_str(), "valid_session");

        let result: Result<SessionId, _> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_operations() {
        let timestamp = Timestamp::from_nanos(1_000_000_000_000);

        assert_eq!(timestamp.as_secs(), 1000);
        assert_eq!(timestamp.as_millis(), 1_000_000);
        assert_eq!(timestamp.as_nanos(), 1_000_000_000_000);
    }

    #[test]
    fn test_timestamp_now() {
        let now1 = Timestamp::now();
        let now2 = Timestamp::now();

        // Should be close in time but could differ
        assert!(now1.as_nanos() <= now2.as_nanos());
    }

    #[test]
    fn test_timestamp_ordering() {
        let ts1 = Timestamp::from_nanos(1000);
        let ts2 = Timestamp::from_nanos(2000);

        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
        assert_eq!(ts1, ts1);
    }

    #[test]
    fn test_timestamp_display() {
        let timestamp = Timestamp::from_nanos(1_640_995_200_000_000_000); // 2022-01-01 00:00:00 UTC
        let display_str = timestamp.to_string();

        // Should contain ISO 8601 format components
        assert!(display_str.contains("2022"));
        assert!(display_str.contains('T'));
        assert!(display_str.contains('Z'));
    }

    #[test]
    fn test_timestamp_conversions() {
        let nanos = 1_500_000_000_000_u64;
        let timestamp: Timestamp = nanos.into();
        assert_eq!(timestamp.as_nanos(), nanos);

        let back_to_nanos: u64 = timestamp.into();
        assert_eq!(back_to_nanos, nanos);
    }

    #[test]
    fn test_newtypes_serde() {
        // Test that newtypes can be serialized/deserialized
        let tool_id = ToolId::new("test_tool").expect("test value should be valid");
        let json = serde_json::to_string(&tool_id).expect("test value should be valid");
        let deserialized: ToolId = serde_json::from_str(&json).expect("test value should be valid");
        assert_eq!(tool_id, deserialized);

        let user_id = UserId::new("test_user").expect("test value should be valid");
        let json = serde_json::to_string(&user_id).expect("test value should be valid");
        let deserialized: UserId = serde_json::from_str(&json).expect("test value should be valid");
        assert_eq!(user_id, deserialized);

        let session_id = SessionId::new("test_session").expect("test value should be valid");
        let json = serde_json::to_string(&session_id).expect("test value should be valid");
        let deserialized: SessionId =
            serde_json::from_str(&json).expect("test value should be valid");
        assert_eq!(session_id, deserialized);

        let timestamp = Timestamp::from_nanos(1_234_567_890);
        let json = serde_json::to_string(&timestamp).expect("test value should be valid");
        let deserialized: Timestamp =
            serde_json::from_str(&json).expect("test value should be valid");
        assert_eq!(timestamp, deserialized);
    }

    #[test]
    fn test_tool_id_edge_cases() {
        // Boundary conditions for length
        let max_length_id = "a".repeat(crate::MAX_TOOL_NAME_LENGTH);
        assert!(ToolId::new(&max_length_id).is_ok());

        let too_long_id = "a".repeat(crate::MAX_TOOL_NAME_LENGTH + 1);
        assert!(ToolId::new(&too_long_id).is_err());

        // Unicode characters should be rejected
        assert!(ToolId::new("tøøl").is_err());
        assert!(ToolId::new("tool🚀").is_err());
    }

    #[test]
    fn test_user_id_edge_cases() {
        // Boundary conditions for length
        let max_length_id = "a".repeat(255);
        assert!(UserId::new(&max_length_id).is_ok());

        let too_long_id = "a".repeat(256);
        assert!(UserId::new(&too_long_id).is_err());

        // Unicode should be allowed in user IDs (more flexible than tool IDs)
        assert!(UserId::new("user_ñame").is_ok());
    }

    #[test]
    fn test_session_id_edge_cases() {
        // Boundary conditions for length
        let max_length_id = "a".repeat(128);
        assert!(SessionId::new(&max_length_id).is_ok());

        let too_long_id = "a".repeat(129);
        assert!(SessionId::new(&too_long_id).is_err());
    }

    #[test]
    fn test_timestamp_edge_cases() {
        // Test with zero
        let zero_ts = Timestamp::from_nanos(0);
        assert_eq!(zero_ts.as_secs(), 0);
        assert_eq!(zero_ts.as_millis(), 0);

        // Test with maximum value
        let max_ts = Timestamp::from_nanos(u64::MAX);
        assert_eq!(max_ts.as_nanos(), u64::MAX);

        // Test ordering with edge values
        assert!(zero_ts < max_ts);
    }

    // Property-based testing placeholder - will be extended with proptest
    #[test]
    fn test_newtype_invariants() {
        // Tool IDs should always be non-empty when created successfully
        for name in ["a", "tool1", "ns.tool", "complex-tool_123"] {
            let tool_id = ToolId::new(name).expect("test value should be valid");
            assert!(!tool_id.as_str().is_empty());
            assert_eq!(tool_id.as_str(), name);
        }

        // User IDs should always be non-empty when created successfully
        for name in ["u", "user1", "principal-id"] {
            let user_id = UserId::new(name).expect("test value should be valid");
            assert!(!user_id.as_str().is_empty());
            assert_eq!(user_id.as_str(), name);
        }

        // Session IDs should always be non-empty when created successfully
        for name in ["s", "session1", "sess_123"] {
            let session_id = SessionId::new(name).expect("test value should be valid");
            assert!(!session_id.as_str().is_empty());
            assert_eq!(session_id.as_str(), name);
        }
    }
}
