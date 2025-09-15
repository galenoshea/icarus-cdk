# Icarus MCP

[![Crates.io](https://img.shields.io/crates/v/icarus-mcp)](https://crates.io/crates/icarus-mcp)
[![Documentation](https://docs.rs/icarus-mcp/badge.svg)](https://docs.rs/icarus-mcp)
[![License](https://img.shields.io/badge/license-BSL--1.1-blue.svg)](../../LICENSE)

A high-performance Model Context Protocol (MCP) server for Internet Computer Protocol (ICP) canisters, featuring advanced optimizations including SIMD acceleration, zero-copy serialization, custom allocators, and streaming responses.

## Features

This crate provides several optional features for modular compilation and performance optimization:

### Default Features

```toml
[dependencies]
icarus-mcp = "0.7.0"
```

Includes: `client`, `server`, `streaming`

### Core Features

- **`client`**: ICP canister client functionality with connection pooling
- **`server`**: Full MCP server implementation with type-state safety
- **`streaming`**: Large response streaming with configurable buffer sizes using const generics
- **`protocol`**: MCP protocol handling and translation with trait-based abstractions

### Advanced Performance Features

- **`simd`**: SIMD-accelerated operations for data processing (x86_64 with AVX2/SSE2 support)
- **`storage`**: Custom allocators, buffer pools, and zero-copy serialization
- **`networking`**: Connection pooling and HTTP transport optimization with FxHashMap performance optimizations

### Development Features

- **`cli`**: Command-line interface utilities for development and testing

### Feature Combinations

```toml
# Minimal client-only setup
[dependencies]
icarus-mcp = { version = "0.7.0", default-features = false, features = ["client"] }

# Server with performance optimizations
[dependencies]
icarus-mcp = { version = "0.7.0", features = ["server", "streaming", "simd", "storage"] }

# Full feature set including CLI tools
[dependencies]
icarus-mcp = { version = "0.7.0", features = ["all"] }

# Maximum performance configuration
[dependencies]
icarus-mcp = { version = "0.7.0", features = ["server", "streaming", "simd", "storage"] }
```

## Performance Optimizations

### SIMD Acceleration (x86_64)

When the `simd` feature is enabled, the crate provides hardware-accelerated operations:

- **Memory Copy**: AVX2/SSE2 optimized bulk data copying
- **Checksums**: Vectorized checksum computation
- **Pattern Matching**: SIMD-accelerated string search
- **JSON Validation**: Fast structural validation

Performance gains: 2-4x speedup for large data operations (>1KB).

### Zero-Copy Serialization

The `storage` feature enables efficient serialization with minimal allocations:

- **Bytes-based sharing**: Reference-counted data sharing
- **Reusable buffers**: Pre-allocated serialization buffers
- **Memory mapping**: Efficient large data handling

### Custom Allocators

Thread-local buffer pools reduce allocation overhead:

- **Pool Management**: Size-specific buffer pools
- **Statistics Tracking**: Allocation pattern monitoring
- **RAII Wrappers**: Automatic buffer return to pools

### Streaming Responses

Configurable buffer sizes using const generics for zero-cost abstraction:

```rust
use icarus_mcp::storage::streaming::{StreamingResponse, Small, Large, CustomSize};

// Small buffers (4KB) for low-latency responses
let small_response = StreamingResponse::<Small>::new();

// Large buffers (256KB) for high-throughput operations
let large_response = StreamingResponse::<Large>::new();

// Custom buffer sizes
let custom_response = StreamingResponse::<CustomSize<128000>>::new();
```

## Quick Start

### Basic MCP Server

```rust
use icarus_mcp::{McpConfig, McpServer};
use candid::Principal;
use std::str::FromStr;
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let canister_id = Principal::from_str("rdmx6-jaaaa-aaaaa-aaadq-cai")?;
    let config = McpConfig::local(canister_id);

    let server = McpServer::from_config(config).await?;
    let serving_server = server.serve(stdin(), stdout()).await?;
    serving_server.run().await?;

    Ok(())
}
```

### Performance-Optimized Server

```rust
use icarus_mcp::{McpConfig, McpServer};
use icarus_mcp::storage::{
    streaming::{StreamingResponse, Large},
    allocator::get_pooled_buffer,
    zerocopy::ZeroCopySerializer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable all performance features
    let config = McpConfig::local(canister_id)
        .with_timeout(30)
        .with_max_concurrent_requests(10);

    // Use pooled allocations for better performance
    let buffer = get_pooled_buffer(64 * 1024);

    // Use streaming responses for large data
    let mut response = StreamingResponse::<Large>::new();

    // Zero-copy serialization
    let mut serializer = ZeroCopySerializer::new().compact();

    Ok(())
}
```

## Examples

The crate includes several examples demonstrating different features:

- **[basic_server.rs](examples/basic_server.rs)** - Simple MCP server setup
- **[streaming.rs](examples/streaming.rs)** - Streaming responses with different buffer sizes
- **[custom_allocator.rs](examples/custom_allocator.rs)** - Buffer pool usage and performance tracking
- **[performance.rs](examples/performance.rs)** - Comprehensive performance benchmarking

Run examples with:

```bash
# Basic server
cargo run --example basic_server

# Streaming with all optimizations
cargo run --example streaming --features=all --release

# Performance benchmarks
cargo run --example performance --features=all --release
```

## Architecture

### Type-State Pattern

The server uses compile-time state tracking for safety:

```rust
// States: Uninitialized -> Connected -> Serving
let server = McpServer::new()           // Uninitialized
    .connect(config).await?             // Connected
    .serve(stdin(), stdout()).await?    // Serving
    .run().await?;                      // Running
```

### Trait-Based Abstractions

Core functionality is abstracted through traits for testability:

- `McpProtocol`: Protocol operations
- `ToolConverter`: Tool metadata conversion
- `CanisterBackend`: ICP integration

### Memory Management

Advanced memory management with multiple strategies:

- **Thread-local pools**: Zero-contention buffer allocation
- **Tracking allocators**: Performance monitoring and debugging
- **Zero-copy operations**: Minimize data copying with reference counting

## Performance Benchmarks

Typical performance characteristics (benchmarked on x86_64):

| Operation | Standard | Optimized | Speedup |
|-----------|----------|-----------|---------|
| Memory Copy (1MB) | 2.1ms | 0.7ms | 3.0x |
| JSON Serialization | 1.2ms | 0.8ms | 1.5x |
| Buffer Allocation | 45μs | 12μs | 3.8x |
| Streaming (10MB) | 15.2ms | 11.1ms | 1.4x |

*Results may vary based on data size, hardware, and compiler optimizations.*

## Requirements

- **Rust**: 1.70+ (for const generics and async traits)
- **SIMD**: x86_64 with AVX2/SSE2 support (automatic fallback to scalar operations)
- **Platform**: Cross-platform (optimizations available on x86_64)

## Development

### Testing

```bash
# Run all tests
cargo test --all-features

# Run performance benchmarks
cargo test --features=all --release -- bench

# Test specific features
cargo test --features=streaming,simd
```

### Documentation

```bash
# Generate documentation
cargo doc --all-features --open

# Check for documentation warnings
cargo doc --no-deps
```

## License

Licensed under the Business Source License 1.1 (BSL). See [LICENSE](../../LICENSE) for details.