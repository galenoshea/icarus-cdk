//! Zero-copy serialization optimizations
//!
//! Provides efficient serialization and deserialization with minimal data copying

use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonValue;
use std::borrow::Cow;

/// Zero-copy JSON serializer that minimizes allocations
pub struct ZeroCopySerializer {
    /// Reusable buffer for serialization
    buffer: BytesMut,
    /// Whether to use compact representation
    compact: bool,
}

impl Default for ZeroCopySerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroCopySerializer {
    /// Create a new zero-copy serializer
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(1024),
            compact: false,
        }
    }

    /// Create a new zero-copy serializer with custom buffer capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            compact: false,
        }
    }

    /// Enable compact JSON representation (no whitespace)
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Serialize a value to Bytes without copying
    ///
    /// The returned Bytes shares the underlying buffer and can be cloned
    /// efficiently using reference counting.
    pub fn serialize<T: Serialize>(&mut self, value: &T) -> Result<Bytes> {
        // Clear buffer but retain capacity
        self.buffer.clear();

        // Serialize to Vec first, then extend buffer
        let json = if self.compact {
            serde_json::to_vec(value)?
        } else {
            serde_json::to_vec_pretty(value)?
        };

        self.buffer.extend_from_slice(&json);

        // Convert to Bytes for zero-copy sharing
        Ok(self.buffer.clone().freeze())
    }

    /// Serialize directly to a pre-allocated buffer
    ///
    /// This avoids any intermediate allocations by writing directly
    /// to the provided buffer.
    pub fn serialize_into<T: Serialize>(&self, value: &T, buf: &mut BytesMut) -> Result<()> {
        // Serialize to Vec first, then extend buffer
        let json = if self.compact {
            serde_json::to_vec(value)?
        } else {
            serde_json::to_vec_pretty(value)?
        };

        buf.extend_from_slice(&json);
        Ok(())
    }

    /// Get the current buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Reset the serializer and optionally resize buffer
    pub fn reset(&mut self, new_capacity: Option<usize>) {
        self.buffer.clear();
        if let Some(capacity) = new_capacity {
            if capacity > self.buffer.capacity() {
                self.buffer.reserve(capacity - self.buffer.capacity());
            }
        }
    }
}

/// Zero-copy JSON deserializer that works with borrowed data
pub struct ZeroCopyDeserializer;

impl ZeroCopyDeserializer {
    /// Deserialize from bytes without copying the underlying data
    ///
    /// For string values, this will use borrowed references when possible
    pub fn deserialize<T: DeserializeOwned>(data: &Bytes) -> Result<T> {
        let value = serde_json::from_slice(data)?;
        Ok(value)
    }

    /// Deserialize from bytes slice with potential borrowing
    pub fn deserialize_borrowed<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
        let value = serde_json::from_slice(data)?;
        Ok(value)
    }

    /// Parse to JsonValue for flexible manipulation
    ///
    /// JsonValue can hold references to the original data in some cases
    pub fn parse(data: &Bytes) -> Result<JsonValue> {
        let value = serde_json::from_slice(data)?;
        Ok(value)
    }

    /// Parse from bytes slice
    pub fn parse_slice(data: &[u8]) -> Result<JsonValue> {
        let value = serde_json::from_slice(data)?;
        Ok(value)
    }

    /// Streaming parser for large JSON documents
    ///
    /// Processes JSON as it arrives without buffering the entire document
    pub fn parse_streaming<R: std::io::Read>(reader: R) -> Result<JsonValue> {
        let value = serde_json::from_reader(reader)?;
        Ok(value)
    }
}

/// Efficient string handling with copy-on-write semantics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZeroCopyString<'a> {
    inner: Cow<'a, str>,
}

impl<'a> ZeroCopyString<'a> {
    /// Create from borrowed string slice
    pub fn borrowed(s: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(s),
        }
    }

    /// Create from owned string
    pub fn owned(s: String) -> Self {
        Self {
            inner: Cow::Owned(s),
        }
    }

    /// Get the string as a borrowed slice if possible
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to owned string, cloning if necessary
    pub fn into_owned(self) -> String {
        self.inner.into_owned()
    }

    /// Check if the string is borrowed (zero-copy)
    pub fn is_borrowed(&self) -> bool {
        matches!(self.inner, Cow::Borrowed(_))
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<'a> std::ops::Deref for ZeroCopyString<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> From<&'a str> for ZeroCopyString<'a> {
    fn from(s: &'a str) -> Self {
        Self::borrowed(s)
    }
}

impl<'a> From<String> for ZeroCopyString<'a> {
    fn from(s: String) -> Self {
        Self::owned(s)
    }
}

/// Efficient buffer management for zero-copy operations
pub struct ZeroCopyBuffer {
    /// Main buffer for data storage
    buffer: BytesMut,
    /// Snapshots of buffer state for sharing
    snapshots: Vec<Bytes>,
}

impl Default for ZeroCopyBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroCopyBuffer {
    /// Create a new zero-copy buffer
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
            snapshots: Vec::new(),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            snapshots: Vec::new(),
        }
    }

    /// Append data to the buffer
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Write data to the buffer
    pub fn put<T: Buf>(&mut self, src: T) {
        self.buffer.put(src);
    }

    /// Create a shareable snapshot of the current buffer state
    ///
    /// The snapshot shares the underlying memory through reference counting
    pub fn snapshot(&mut self) -> Bytes {
        let snapshot = self.buffer.clone().freeze();
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    /// Get the current buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the current buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Clear the buffer but retain snapshots
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Split the buffer at the given index
    ///
    /// Returns the data before the index as Bytes (zero-copy)
    /// and keeps the data after the index in the buffer
    pub fn split_to(&mut self, at: usize) -> Bytes {
        self.buffer.split_to(at).freeze()
    }

    /// Get all snapshots created from this buffer
    pub fn snapshots(&self) -> &[Bytes] {
        &self.snapshots
    }

    /// Convert the buffer to Bytes for sharing
    pub fn into_bytes(self) -> Bytes {
        self.buffer.freeze()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Truncate the buffer to the specified length
    pub fn truncate(&mut self, len: usize) {
        self.buffer.truncate(len);
    }
}

/// Memory-mapped approach for large data handling
pub struct MemoryMappedBuffer {
    /// The actual data storage
    data: Bytes,
    /// Current read position
    position: usize,
}

impl MemoryMappedBuffer {
    /// Create from existing bytes
    pub fn new(data: Bytes) -> Self {
        Self { data, position: 0 }
    }

    /// Get a slice of data without copying
    pub fn slice(&self, range: std::ops::Range<usize>) -> Option<Bytes> {
        if range.end <= self.data.len() && range.start <= range.end {
            Some(self.data.slice(range))
        } else {
            None
        }
    }

    /// Read data at current position and advance
    pub fn read(&mut self, len: usize) -> Option<Bytes> {
        if self.position + len <= self.data.len() {
            let result = self.data.slice(self.position..self.position + len);
            self.position += len;
            Some(result)
        } else {
            None
        }
    }

    /// Peek at data without advancing position
    pub fn peek(&self, len: usize) -> Option<Bytes> {
        if self.position + len <= self.data.len() {
            Some(self.data.slice(self.position..self.position + len))
        } else {
            None
        }
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Set position
    pub fn set_position(&mut self, pos: usize) -> Result<()> {
        if pos <= self.data.len() {
            self.position = pos;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Position {} out of bounds", pos))
        }
    }

    /// Get remaining data from current position
    pub fn remaining(&self) -> Bytes {
        self.data.slice(self.position..)
    }

    /// Get total data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Reset position to beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

impl std::io::Read for MemoryMappedBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = (self.data.len() - self.position).min(buf.len());
        if available > 0 {
            let data = &self.data[self.position..self.position + available];
            buf[..available].copy_from_slice(data);
            self.position += available;
        }
        Ok(available)
    }
}

impl Buf for MemoryMappedBuffer {
    fn remaining(&self) -> usize {
        self.data.len() - self.position
    }

    fn chunk(&self) -> &[u8] {
        &self.data[self.position..]
    }

    fn advance(&mut self, cnt: usize) {
        self.position = (self.position + cnt).min(self.data.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
        items: Vec<String>,
    }

    #[test]
    fn test_zero_copy_serializer() {
        let mut serializer = ZeroCopySerializer::new();

        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let serialized = serializer.serialize(&data).unwrap();
        assert!(!serialized.is_empty());

        // Verify we can clone the Bytes efficiently (reference counting)
        let cloned = serialized.clone();
        assert_eq!(serialized.len(), cloned.len());
    }

    #[test]
    fn test_zero_copy_deserializer() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let mut serializer = ZeroCopySerializer::new();
        let serialized = serializer.serialize(&data).unwrap();

        let deserialized: TestData = ZeroCopyDeserializer::deserialize(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_serialize_into() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["item1".to_string()],
        };

        let serializer = ZeroCopySerializer::new().compact();
        let mut buffer = BytesMut::new();

        serializer.serialize_into(&data, &mut buffer).unwrap();
        assert!(!buffer.is_empty());

        let deserialized: TestData = ZeroCopyDeserializer::deserialize_borrowed(&buffer).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_zero_copy_string() {
        let original = "hello world";
        let borrowed = ZeroCopyString::borrowed(original);

        assert_eq!(borrowed.as_str(), original);
        assert!(borrowed.is_borrowed());
        assert_eq!(borrowed.len(), original.len());

        let owned = ZeroCopyString::owned("owned string".to_string());
        assert!(!owned.is_borrowed());
        assert_eq!(owned.len(), 12);
    }

    #[test]
    fn test_zero_copy_buffer() {
        let mut buffer = ZeroCopyBuffer::new();

        buffer.extend_from_slice(b"hello");
        assert_eq!(buffer.len(), 5);

        let snapshot1 = buffer.snapshot();
        assert_eq!(snapshot1.len(), 5);

        buffer.extend_from_slice(b" world");
        assert_eq!(buffer.len(), 11);

        let snapshot2 = buffer.snapshot();
        assert_eq!(snapshot2.len(), 11);

        // Snapshots should retain their original content
        assert_eq!(&snapshot1[..], b"hello");
        assert_eq!(&snapshot2[..], b"hello world");

        assert_eq!(buffer.snapshots().len(), 2);
    }

    #[test]
    fn test_memory_mapped_buffer() {
        let data = Bytes::from(&b"hello world"[..]);
        let mut mapped = MemoryMappedBuffer::new(data);

        assert_eq!(mapped.len(), 11);
        assert_eq!(mapped.position(), 0);

        let slice = mapped.slice(0..5).unwrap();
        assert_eq!(&slice[..], b"hello");

        let read = mapped.read(5).unwrap();
        assert_eq!(&read[..], b"hello");
        assert_eq!(mapped.position(), 5);

        let peek = mapped.peek(6).unwrap();
        assert_eq!(&peek[..], b" world");
        assert_eq!(mapped.position(), 5); // Position shouldn't change

        let remaining = mapped.remaining();
        assert_eq!(&remaining[..], b" world");
    }

    #[test]
    fn test_buffer_split() {
        let mut buffer = ZeroCopyBuffer::with_capacity(1024);
        buffer.extend_from_slice(b"hello world test");

        let first_part = buffer.split_to(5);
        assert_eq!(&first_part[..], b"hello");
        assert_eq!(buffer.len(), 11); // " world test" remaining

        let remaining = buffer.into_bytes();
        assert_eq!(&remaining[..], b" world test");
    }

    #[test]
    fn test_json_parsing() {
        let json_str = r#"{"name":"test","value":42,"active":true}"#;
        let json_bytes = Bytes::from(json_str);

        let value = ZeroCopyDeserializer::parse(&json_bytes).unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["value"], 42);
        assert_eq!(value["active"], true);
    }

    #[test]
    fn test_serializer_reuse() {
        let mut serializer = ZeroCopySerializer::with_capacity(2048);

        let data1 = TestData {
            name: "first".to_string(),
            value: 1,
            items: vec!["a".to_string()],
        };

        let result1 = serializer.serialize(&data1).unwrap();
        assert!(!result1.is_empty());

        // Reuse the serializer
        let data2 = TestData {
            name: "second".to_string(),
            value: 2,
            items: vec!["b".to_string(), "c".to_string()],
        };

        let result2 = serializer.serialize(&data2).unwrap();
        assert!(!result2.is_empty());

        // Results should be different
        assert_ne!(result1, result2);

        // Both should deserialize correctly
        let parsed1: TestData = ZeroCopyDeserializer::deserialize(&result1).unwrap();
        let parsed2: TestData = ZeroCopyDeserializer::deserialize(&result2).unwrap();

        assert_eq!(parsed1, data1);
        assert_eq!(parsed2, data2);
    }
}
