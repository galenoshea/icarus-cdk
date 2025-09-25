//! Streaming support for large responses
//!
//! Provides efficient handling of large responses without buffering everything in memory

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use serde_json::Value as JsonValue;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, warn};

#[cfg(any(feature = "simd", feature = "streaming"))]
use super::simd::SimdProcessor;

#[cfg(feature = "storage")]
use super::allocator::get_pooled_buffer;

/// Default maximum chunk size for streaming responses (64KB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Small buffer size for small responses (4KB)
pub const SMALL_BUFFER_SIZE: usize = 4 * 1024;

/// Large buffer size for large responses (256KB)
pub const LARGE_BUFFER_SIZE: usize = 256 * 1024;

/// Trait for buffer size configuration
pub trait BufferSize: Unpin + Send + Sync {
    /// The buffer size in bytes
    const SIZE: usize;
}

/// Buffer size marker for small responses
#[derive(Debug, Clone, Copy)]
pub struct Small;

impl BufferSize for Small {
    const SIZE: usize = SMALL_BUFFER_SIZE;
}

/// Buffer size marker for default responses
#[derive(Debug, Clone, Copy)]
pub struct DefaultSize;

impl BufferSize for DefaultSize {
    const SIZE: usize = DEFAULT_CHUNK_SIZE;
}

/// Buffer size marker for large responses
#[derive(Debug, Clone, Copy)]
pub struct Large;

impl BufferSize for Large {
    const SIZE: usize = LARGE_BUFFER_SIZE;
}

/// Custom buffer size with const generic
#[derive(Debug, Clone, Copy)]
pub struct CustomSize<const SIZE: usize>;

impl<const SIZE: usize> BufferSize for CustomSize<SIZE> {
    const SIZE: usize = SIZE;
}

/// Streaming response wrapper with configurable buffer size
///
/// Provides a streaming interface for large canister responses without
/// requiring the entire response to be buffered in memory.
/// The buffer size is configured via const generics for zero-cost abstraction.
pub struct StreamingResponse<B: BufferSize = DefaultSize> {
    /// Current buffer of data
    buffer: BytesMut,
    /// Total bytes read so far
    bytes_read: usize,
    /// Expected total size (if known)
    total_size: Option<usize>,
    /// Whether we've reached the end of the stream
    finished: bool,
    /// Buffer size marker
    _buffer_size: PhantomData<B>,
}

impl<B: BufferSize> Default for StreamingResponse<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: BufferSize> StreamingResponse<B> {
    /// Create a new streaming response with buffer size type
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(B::SIZE),
            bytes_read: 0,
            total_size: None,
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Create a new streaming response using pooled memory allocation
    #[cfg(feature = "storage")]
    pub fn new_pooled() -> Self {
        Self {
            buffer: get_pooled_buffer(B::SIZE),
            bytes_read: 0,
            total_size: None,
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Create a streaming response with known total size
    pub fn with_size(total_size: usize) -> Self {
        let capacity = total_size.min(B::SIZE);
        Self {
            buffer: BytesMut::with_capacity(capacity),
            bytes_read: 0,
            total_size: Some(total_size),
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Create a streaming response with known total size using pooled allocation
    #[cfg(feature = "storage")]
    pub fn with_size_pooled(total_size: usize) -> Self {
        let capacity = total_size.min(B::SIZE);
        let mut buffer = get_pooled_buffer(capacity);
        buffer.reserve(capacity);
        Self {
            buffer,
            bytes_read: 0,
            total_size: Some(total_size),
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Create a streaming response with custom initial capacity
    pub fn with_capacity(initial_capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(initial_capacity),
            bytes_read: 0,
            total_size: None,
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Get the configured buffer size
    #[inline]
    pub const fn buffer_size() -> usize {
        B::SIZE
    }
}

// Backward compatibility implementation
impl StreamingResponse<DefaultSize> {
    /// Create a new streaming response (backward compatibility)
    pub fn new_default(initial_capacity: Option<usize>) -> Self {
        let capacity = initial_capacity.unwrap_or(DEFAULT_CHUNK_SIZE);
        Self {
            buffer: BytesMut::with_capacity(capacity),
            bytes_read: 0,
            total_size: None,
            finished: false,
            _buffer_size: PhantomData,
        }
    }

    /// Create a streaming response with known total size (backward compatibility)
    pub fn with_size_default(total_size: usize) -> Self {
        let capacity = total_size.min(DEFAULT_CHUNK_SIZE);
        Self {
            buffer: BytesMut::with_capacity(capacity),
            bytes_read: 0,
            total_size: Some(total_size),
            finished: false,
            _buffer_size: PhantomData,
        }
    }
}

impl<B: BufferSize> StreamingResponse<B> {
    /// Get the current progress (0.0 to 1.0) if total size is known
    pub fn progress(&self) -> Option<f64> {
        self.total_size.map(|total| {
            if total == 0 {
                1.0
            } else {
                (self.bytes_read as f64) / (total as f64)
            }
        })
    }

    /// Get the bytes read so far
    #[inline]
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    /// Check if the response is finished
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Add data to the buffer
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        self.bytes_read += data.len();
    }

    /// Add data to the buffer with SIMD-optimized copy for large data
    #[cfg(any(feature = "simd", feature = "streaming"))]
    pub fn extend_from_slice_simd(&mut self, data: &[u8]) {
        if data.len() > 1024 {
            // Use SIMD-optimized copy for large data blocks
            let current_len = self.buffer.len();
            self.buffer.resize(current_len + data.len(), 0);

            if SimdProcessor::fast_copy(data, &mut self.buffer[current_len..]).is_err() {
                // Fallback to standard copy on error
                self.buffer.truncate(current_len);
                self.buffer.extend_from_slice(data);
            }
        } else {
            // Use standard copy for small data
            self.buffer.extend_from_slice(data);
        }
        self.bytes_read += data.len();
    }

    /// Get the next chunk of data
    pub fn next_chunk(&mut self) -> Option<Bytes> {
        if self.buffer.is_empty() {
            return None;
        }

        let chunk_size = self.buffer.len().min(B::SIZE);
        let chunk = self.buffer.split_to(chunk_size);
        Some(chunk.freeze())
    }

    /// Mark the response as finished
    pub fn finish(&mut self) {
        self.finished = true;
    }

    /// Try to parse the current buffer as JSON
    ///
    /// This is useful for responses that are JSON but may be streamed in chunks
    pub fn try_parse_json(&self) -> Result<Option<JsonValue>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        match serde_json::from_slice(&self.buffer) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.is_eof() => {
                // Incomplete JSON, need more data
                debug!("Incomplete JSON, waiting for more data");
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to parse JSON: {}", e);
                Err(e.into())
            }
        }
    }

    /// Try to parse JSON with SIMD-accelerated pre-validation
    #[cfg(any(feature = "simd", feature = "streaming"))]
    pub fn try_parse_json_simd(&self) -> Result<Option<JsonValue>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        // Use SIMD to quickly validate JSON structure before parsing
        if !SimdProcessor::validate_json_structure(&self.buffer) {
            return Ok(None); // Structure invalid, likely incomplete
        }

        match serde_json::from_slice(&self.buffer) {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.is_eof() => {
                debug!("Incomplete JSON, waiting for more data");
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to parse JSON: {}", e);
                Err(e.into())
            }
        }
    }

    /// Compute SIMD-accelerated checksum of the buffer
    #[cfg(any(feature = "simd", feature = "streaming"))]
    pub fn checksum(&self) -> u64 {
        SimdProcessor::fast_checksum(&self.buffer)
    }

    /// Find pattern in buffer using SIMD acceleration
    #[cfg(any(feature = "simd", feature = "streaming"))]
    pub fn find_pattern(&self, pattern: &[u8]) -> Option<usize> {
        SimdProcessor::fast_find(&self.buffer, pattern)
    }

    /// Compare buffer contents with another buffer using SIMD
    #[cfg(any(feature = "simd", feature = "streaming"))]
    pub fn fast_equals(&self, other: &[u8]) -> bool {
        SimdProcessor::fast_compare(&self.buffer, other)
    }

    /// Get all remaining data as bytes
    pub fn into_bytes(self) -> Bytes {
        self.buffer.freeze()
    }

    /// Get all data as a string (assumes UTF-8)
    pub fn into_string(self) -> Result<String> {
        let bytes = self.into_bytes();
        String::from_utf8(bytes.to_vec()).map_err(Into::into)
    }
}

impl<B: BufferSize> AsyncRead for StreamingResponse<B> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();

        if this.buffer.is_empty() {
            if this.finished {
                return Poll::Ready(Ok(()));
            } else {
                return Poll::Pending;
            }
        }

        let to_read = buf.remaining().min(this.buffer.len());
        let data = this.buffer.split_to(to_read);
        buf.put_slice(&data);

        Poll::Ready(Ok(()))
    }
}

/// Stream adapter for large responses with configurable buffer size
pub struct ResponseStream<B: BufferSize = DefaultSize> {
    response: StreamingResponse<B>,
}

impl<B: BufferSize> ResponseStream<B> {
    /// Create a new response stream
    pub fn new(response: StreamingResponse<B>) -> Self {
        Self { response }
    }

    /// Get the underlying response
    pub fn into_inner(self) -> StreamingResponse<B> {
        self.response
    }
}

impl<B: BufferSize> Stream for ResponseStream<B> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(chunk) = this.response.next_chunk() {
            Poll::Ready(Some(Ok(chunk)))
        } else if this.response.is_finished() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

/// Utility function to collect a stream into a single buffer
pub async fn collect_stream<S>(mut stream: S) -> Result<Bytes>
where
    S: Stream<Item = Result<Bytes>> + Unpin,
{
    let mut buffer = BytesMut::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);
    }

    Ok(buffer.freeze())
}

/// Utility function to write a stream to an AsyncWrite destination
pub async fn write_stream_to<W, S>(mut writer: W, mut stream: S) -> Result<usize>
where
    W: AsyncWrite + Unpin,
    S: Stream<Item = Result<Bytes>> + Unpin,
{
    use tokio::io::AsyncWriteExt;

    let mut total_written = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk).await?;
        total_written += chunk.len();
    }

    writer.flush().await?;
    Ok(total_written)
}

#[cfg(test)]
mod tests {
    use super::*;
    // tokio_test is available if needed for testing

    #[test]
    fn test_streaming_response_creation() {
        let response = StreamingResponse::<DefaultSize>::new();
        assert_eq!(response.bytes_read(), 0);
        assert!(!response.is_finished());
        assert!(response.progress().is_none());
        assert_eq!(
            StreamingResponse::<DefaultSize>::buffer_size(),
            DEFAULT_CHUNK_SIZE
        );
    }

    #[test]
    fn test_streaming_response_with_size() {
        let response = StreamingResponse::<DefaultSize>::with_size(2048);
        assert_eq!(response.bytes_read(), 0);
        assert!(!response.is_finished());
        assert_eq!(response.progress(), Some(0.0));
    }

    #[test]
    fn test_progress_tracking() {
        let mut response = StreamingResponse::<DefaultSize>::with_size(1000);
        assert_eq!(response.progress(), Some(0.0));

        response.extend_from_slice(b"hello");
        assert_eq!(response.bytes_read(), 5);
        assert_eq!(response.progress(), Some(0.005));

        response.extend_from_slice(&vec![b'x'; 995]);
        assert_eq!(response.bytes_read(), 1000);
        assert_eq!(response.progress(), Some(1.0));
    }

    #[test]
    fn test_chunk_reading() {
        let mut response = StreamingResponse::<Small>::new();
        response.extend_from_slice(b"hello world");

        let chunk = response.next_chunk();
        assert!(chunk.is_some());
        assert_eq!(chunk.unwrap(), "hello world");

        let empty_chunk = response.next_chunk();
        assert!(empty_chunk.is_none());
    }

    #[test]
    fn test_json_parsing() {
        let mut response = StreamingResponse::<DefaultSize>::new();

        // Incomplete JSON
        response.extend_from_slice(b"{\"key\": \"val");
        assert!(response.try_parse_json().unwrap().is_none());

        // Complete JSON
        response.extend_from_slice(b"ue\"}");
        let json = response.try_parse_json().unwrap();
        assert!(json.is_some());
        assert_eq!(json.unwrap()["key"], "value");
    }

    #[test]
    fn test_const_generic_buffer_sizes() {
        // Test different buffer size configurations
        assert_eq!(StreamingResponse::<Small>::buffer_size(), SMALL_BUFFER_SIZE);
        assert_eq!(
            StreamingResponse::<DefaultSize>::buffer_size(),
            DEFAULT_CHUNK_SIZE
        );
        assert_eq!(StreamingResponse::<Large>::buffer_size(), LARGE_BUFFER_SIZE);
        assert_eq!(StreamingResponse::<CustomSize<1024>>::buffer_size(), 1024);
        assert_eq!(
            StreamingResponse::<CustomSize<512000>>::buffer_size(),
            512000
        );

        // Test creation with different buffer sizes
        let small_response = StreamingResponse::<Small>::new();
        let default_response = StreamingResponse::<DefaultSize>::new();
        let large_response = StreamingResponse::<Large>::new();
        let custom_response = StreamingResponse::<CustomSize<2048>>::new();

        assert_eq!(small_response.bytes_read(), 0);
        assert_eq!(default_response.bytes_read(), 0);
        assert_eq!(large_response.bytes_read(), 0);
        assert_eq!(custom_response.bytes_read(), 0);
    }

    #[tokio::test]
    async fn test_response_stream() {
        let mut response = StreamingResponse::<Small>::new();
        response.extend_from_slice(b"test data");
        response.finish();

        let mut stream = ResponseStream::new(response);
        let chunk = stream.next().await;
        assert!(chunk.is_some());
        assert_eq!(chunk.unwrap().unwrap(), "test data");

        let end = stream.next().await;
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn test_collect_stream() {
        let mut response = StreamingResponse::<DefaultSize>::new();
        response.extend_from_slice(b"chunk1");
        response.extend_from_slice(b"chunk2");
        response.finish();

        let stream = ResponseStream::new(response);
        let collected = collect_stream(stream).await.unwrap();
        assert_eq!(collected, "chunk1chunk2");
    }

    #[test]
    fn test_backward_compatibility() {
        // Test backward compatibility methods
        let response = StreamingResponse::<DefaultSize>::new_default(Some(1024));
        assert_eq!(response.bytes_read(), 0);

        let response = StreamingResponse::<DefaultSize>::with_size_default(2048);
        assert_eq!(response.bytes_read(), 0);
        assert_eq!(response.progress(), Some(0.0));
    }
}
