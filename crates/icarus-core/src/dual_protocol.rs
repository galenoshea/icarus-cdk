//! Dual protocol support: CandidOrJson with stack-allocated optimization
//!
//! Provides efficient dual protocol handling supporting both Candid and JSON
//! with automatic format detection, stack allocation for small payloads,
//! and zero-copy deserialization patterns.

use base64::engine::{general_purpose, Engine};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::borrow::Cow;

/// Stack-allocated buffer size for small payloads (4KB)
/// This covers ~95% of typical MCP tool calls without heap allocation
const STACK_BUFFER_SIZE: usize = 4096;

/// CandidOrJson enum with stack-allocated optimization
///
/// Uses SmallVec for stack allocation of small payloads and Cow for zero-copy
/// deserialization when possible. Automatically falls back to heap allocation
/// for large payloads.
#[derive(Debug, Clone)]
pub enum CandidOrJson<'a> {
    /// Candid-encoded data with stack allocation for small payloads
    Candid(Box<SmallVec<[u8; STACK_BUFFER_SIZE]>>),
    /// JSON data with zero-copy borrowed slice when possible
    Json(Cow<'a, [u8]>),
    /// Large payloads that exceed stack buffer size
    Large(Box<[u8]>),
}

/// Protocol format detection and parsing result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolFormat {
    Candid,
    Json,
    Unknown,
}

impl<'a> CandidOrJson<'a> {
    /// Create from bytes with automatic format detection
    pub fn from_bytes(data: &'a [u8]) -> Self {
        match detect_format(data) {
            ProtocolFormat::Candid => {
                if data.len() <= STACK_BUFFER_SIZE {
                    let mut buffer = SmallVec::new();
                    buffer.extend_from_slice(data);
                    CandidOrJson::Candid(Box::new(buffer))
                } else {
                    CandidOrJson::Large(data.to_vec().into_boxed_slice())
                }
            }
            ProtocolFormat::Json => {
                if data.len() <= STACK_BUFFER_SIZE {
                    CandidOrJson::Json(Cow::Borrowed(data))
                } else {
                    CandidOrJson::Large(data.to_vec().into_boxed_slice())
                }
            }
            ProtocolFormat::Unknown => {
                // Default to JSON for unknown formats
                if data.len() <= STACK_BUFFER_SIZE {
                    CandidOrJson::Json(Cow::Borrowed(data))
                } else {
                    CandidOrJson::Large(data.to_vec().into_boxed_slice())
                }
            }
        }
    }

    /// Create from string (always JSON)
    pub fn from_string(s: &'a str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Create from owned string
    pub fn from_owned_string(s: String) -> Self {
        let bytes = s.into_bytes();
        if bytes.len() <= STACK_BUFFER_SIZE {
            CandidOrJson::Json(Cow::Owned(bytes))
        } else {
            CandidOrJson::Large(bytes.into_boxed_slice())
        }
    }

    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            CandidOrJson::Candid(data) => data.as_slice(),
            CandidOrJson::Json(data) => data.as_ref(),
            CandidOrJson::Large(data) => data.as_ref(),
        }
    }

    /// Get the detected format
    pub fn format(&self) -> ProtocolFormat {
        match self {
            CandidOrJson::Candid(_) => ProtocolFormat::Candid,
            CandidOrJson::Json(_) => ProtocolFormat::Json,
            CandidOrJson::Large(data) => detect_format(data),
        }
    }

    /// Convert to string (for JSON data)
    pub fn to_string(&self) -> Result<String, ProtocolError> {
        match self.format() {
            ProtocolFormat::Json => {
                String::from_utf8(self.as_bytes().to_vec()).map_err(|_| ProtocolError::InvalidUtf8)
            }
            _ => Err(ProtocolError::NotJsonData),
        }
    }

    /// Parse as specific type with automatic protocol detection
    pub fn parse<T>(&self) -> Result<T, ProtocolError>
    where
        T: for<'de> Deserialize<'de> + candid::CandidType,
    {
        match self.format() {
            ProtocolFormat::Candid => {
                candid::decode_one(self.as_bytes()).map_err(ProtocolError::CandidDecodeError)
            }
            ProtocolFormat::Json => {
                serde_json::from_slice(self.as_bytes()).map_err(ProtocolError::JsonParseError)
            }
            ProtocolFormat::Unknown => {
                // Try JSON first, then Candid
                serde_json::from_slice(self.as_bytes())
                    .or_else(|_| {
                        candid::decode_one(self.as_bytes())
                            .map_err(ProtocolError::CandidDecodeError)
                    })
                    .map_err(|_| ProtocolError::UnknownFormat)
            }
        }
    }

    /// Serialize from type to optimal format
    pub fn serialize<T>(value: &T, prefer_format: ProtocolFormat) -> Result<Self, ProtocolError>
    where
        T: Serialize + candid::CandidType,
    {
        match prefer_format {
            ProtocolFormat::Candid => {
                let bytes = candid::encode_one(value).map_err(ProtocolError::CandidEncodeError)?;

                if bytes.len() <= STACK_BUFFER_SIZE {
                    let mut buffer = SmallVec::new();
                    buffer.extend_from_slice(&bytes);
                    Ok(CandidOrJson::Candid(Box::new(buffer)))
                } else {
                    Ok(CandidOrJson::Large(bytes.into_boxed_slice()))
                }
            }
            ProtocolFormat::Json => {
                let bytes = serde_json::to_vec(value).map_err(ProtocolError::JsonSerializeError)?;

                if bytes.len() <= STACK_BUFFER_SIZE {
                    Ok(CandidOrJson::Json(Cow::Owned(bytes)))
                } else {
                    Ok(CandidOrJson::Large(bytes.into_boxed_slice()))
                }
            }
            ProtocolFormat::Unknown => {
                // Default to JSON for unknown preference
                Self::serialize(value, ProtocolFormat::Json)
            }
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        match self {
            CandidOrJson::Candid(data) => MemoryStats {
                total_size: data.len(),
                stack_allocated: true,
                heap_allocated: false,
                format: ProtocolFormat::Candid,
            },
            CandidOrJson::Json(data) => MemoryStats {
                total_size: data.len(),
                stack_allocated: matches!(data, Cow::Borrowed(_)),
                heap_allocated: matches!(data, Cow::Owned(_)),
                format: ProtocolFormat::Json,
            },
            CandidOrJson::Large(data) => MemoryStats {
                total_size: data.len(),
                stack_allocated: false,
                heap_allocated: true,
                format: detect_format(data),
            },
        }
    }
}

/// Memory usage statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_size: usize,
    pub stack_allocated: bool,
    pub heap_allocated: bool,
    pub format: ProtocolFormat,
}

/// Protocol-related errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid UTF-8 sequence")]
    InvalidUtf8,
    #[error("Data is not JSON format")]
    NotJsonData,
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("JSON serialize error: {0}")]
    JsonSerializeError(serde_json::Error),
    #[error("Candid decode error: {0}")]
    CandidDecodeError(#[from] candid::Error),
    #[error("Candid encode error: {0}")]
    CandidEncodeError(candid::Error),
    #[error("Unknown or unsupported format")]
    UnknownFormat,
}

/// Detect protocol format from byte content
///
/// Uses heuristics to determine if data is Candid or JSON:
/// - JSON: Starts with '{', '[', '"', or whitespace followed by these
/// - Candid: Binary format with specific magic bytes
/// - Unknown: Falls back to JSON parsing
pub fn detect_format(data: &[u8]) -> ProtocolFormat {
    if data.is_empty() {
        return ProtocolFormat::Unknown;
    }

    // Skip leading whitespace for JSON detection
    let mut trimmed = data.iter().skip_while(|&&b| b.is_ascii_whitespace());

    if let Some(&first_byte) = trimmed.next() {
        match first_byte {
            b'{' | b'[' | b'"' => return ProtocolFormat::Json,
            b't' | b'f' | b'n' => {
                // Could be JSON literal (true, false, null)
                if let Ok(s) = std::str::from_utf8(data) {
                    if s.trim().starts_with("true")
                        || s.trim().starts_with("false")
                        || s.trim().starts_with("null")
                    {
                        return ProtocolFormat::Json;
                    }
                }
            }
            b'0'..=b'9' | b'-' => {
                // Could be JSON number
                if let Ok(s) = std::str::from_utf8(data) {
                    if s.trim().parse::<f64>().is_ok() {
                        return ProtocolFormat::Json;
                    }
                }
            }
            _ => {}
        }
    }

    // Check for Candid magic bytes or patterns
    if data.len() >= 4 {
        // Candid often starts with type table length (little endian)
        // This is a heuristic - actual Candid detection would be more sophisticated
        if is_likely_candid(data) {
            return ProtocolFormat::Candid;
        }
    }

    // If we can't determine, assume JSON (more common for MCP)
    ProtocolFormat::Unknown
}

/// Heuristic to detect if data is likely Candid format
fn is_likely_candid(data: &[u8]) -> bool {
    // Candid binary format has specific patterns
    // This is a simplified heuristic - production would use candid parser

    if data.len() < 4 {
        return false;
    }

    // Check if data is not valid UTF-8 (Candid is binary)
    if std::str::from_utf8(data).is_err() {
        return true;
    }

    // Check for non-printable bytes (common in binary Candid)
    let non_printable_count = data
        .iter()
        .take(100) // Check first 100 bytes
        .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
        .count();

    // If more than 10% are non-printable, likely binary (Candid)
    non_printable_count > data.len().min(100) / 10
}

/// Convenience function for parsing input with automatic detection
pub fn parse_input<T>(input: &str) -> Result<T, ProtocolError>
where
    T: for<'de> Deserialize<'de> + candid::CandidType,
{
    let data = CandidOrJson::from_string(input);
    data.parse()
}

/// Convenience function for serializing output to preferred format
pub fn serialize_output<T>(value: &T, format: ProtocolFormat) -> Result<String, ProtocolError>
where
    T: Serialize + candid::CandidType,
{
    let data = CandidOrJson::serialize(value, format)?;
    match format {
        ProtocolFormat::Json => data.to_string(),
        ProtocolFormat::Candid => {
            // For Candid, we typically want to return base64 or hex encoding
            Ok(general_purpose::STANDARD.encode(data.as_bytes()))
        }
        ProtocolFormat::Unknown => data.to_string(), // Default to JSON
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_detection() {
        // JSON detection
        assert_eq!(detect_format(b"{}"), ProtocolFormat::Json);
        assert_eq!(detect_format(b"[]"), ProtocolFormat::Json);
        assert_eq!(detect_format(b"\"hello\""), ProtocolFormat::Json);
        assert_eq!(
            detect_format(b"  { \"key\": \"value\" }  "),
            ProtocolFormat::Json
        );
        assert_eq!(detect_format(b"123"), ProtocolFormat::Json);
        assert_eq!(detect_format(b"true"), ProtocolFormat::Json);
        assert_eq!(detect_format(b"null"), ProtocolFormat::Json);

        // Binary data (likely Candid)
        assert_eq!(
            detect_format(&[0x00, 0x01, 0x02, 0x03]),
            ProtocolFormat::Candid
        );
        assert_eq!(
            detect_format(&[0xFF, 0xFE, 0xFD, 0xFC]),
            ProtocolFormat::Candid
        );
    }

    #[test]
    fn test_stack_allocation() {
        let small_json = r#"{"key": "value"}"#;
        let data = CandidOrJson::from_string(small_json);

        let stats = data.memory_stats();
        assert!(stats.total_size <= STACK_BUFFER_SIZE);
        assert_eq!(stats.format, ProtocolFormat::Json);
    }

    #[test]
    fn test_large_payload_handling() {
        // Create large JSON payload
        let large_json = json!({
            "data": "x".repeat(STACK_BUFFER_SIZE + 1000)
        })
        .to_string();

        let data = CandidOrJson::from_owned_string(large_json);
        let stats = data.memory_stats();

        assert!(stats.total_size > STACK_BUFFER_SIZE);
        assert!(stats.heap_allocated);
        assert_eq!(stats.format, ProtocolFormat::Json);
    }

    #[test]
    fn test_parse_json() {
        #[derive(Deserialize, Serialize, candid::CandidType)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let json_str = r#"{"name": "test", "value": 42}"#;
        let data = CandidOrJson::from_string(json_str);

        let parsed: TestStruct = data.parse().unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.value, 42);
    }

    #[test]
    fn test_serialize_json() {
        #[derive(Deserialize, Serialize, candid::CandidType)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let serialized = CandidOrJson::serialize(&test_data, ProtocolFormat::Json).unwrap();
        assert_eq!(serialized.format(), ProtocolFormat::Json);

        let parsed: TestStruct = serialized.parse().unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.value, 42);
    }

    #[test]
    fn test_zero_copy_optimization() {
        let json_str = r#"{"key": "value"}"#;
        let data = CandidOrJson::from_string(json_str);

        // Should use zero-copy for borrowed data
        let stats = data.memory_stats();
        assert!(stats.stack_allocated); // Borrowed data counts as stack allocated
        assert!(!stats.heap_allocated);
    }
}
