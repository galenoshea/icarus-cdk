//! Storage module for efficient data handling
//!
//! This module provides storage and streaming capabilities:
//! - Configurable streaming responses for large data
//! - Zero-cost buffer size abstractions
//! - Memory-efficient data processing
//! - SIMD-accelerated data operations
//! - Custom allocators for high-frequency operations

#[cfg(all(feature = "mcp", feature = "streaming"))]
pub mod streaming;

#[cfg(all(feature = "mcp", any(feature = "simd", feature = "streaming")))]
pub mod simd;

#[cfg(all(feature = "mcp", feature = "storage"))]
pub mod allocator;

#[cfg(all(feature = "mcp", feature = "storage"))]
pub mod zerocopy;

#[cfg(all(feature = "mcp", feature = "storage"))]
pub mod profile;

#[cfg(all(feature = "mcp", feature = "streaming"))]
pub use streaming::{
    collect_stream, write_stream_to, BufferSize, CustomSize, DefaultSize as DefaultBuffer, Large,
    ResponseStream, Small, StreamingResponse, DEFAULT_CHUNK_SIZE, LARGE_BUFFER_SIZE,
    SMALL_BUFFER_SIZE,
};

#[cfg(all(feature = "mcp", any(feature = "simd", feature = "streaming")))]
pub use simd::SimdProcessor;

#[cfg(all(feature = "mcp", feature = "storage"))]
pub use allocator::{
    get_pool_stats, get_pooled_buffer, return_pooled_buffer, BufferPool, PooledBuffer,
    TrackingAllocator,
};

#[cfg(all(feature = "mcp", feature = "storage"))]
pub use zerocopy::{
    MemoryMappedBuffer, ZeroCopyBuffer, ZeroCopyDeserializer, ZeroCopySerializer, ZeroCopyString,
};

#[cfg(all(feature = "mcp", feature = "storage"))]
pub use profile::{
    thread_profiler, AllocationReport, FunctionReport, HotPathReport, OptimizationHint,
    OptimizationType, PerformanceImpact, PerformanceProfiler, PerformanceReport, TimingGuard,
};
