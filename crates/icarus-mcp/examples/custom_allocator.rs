//! Custom Allocator Example
//!
//! Demonstrates how to use the custom buffer pool allocator system
//! for high-performance memory management in MCP applications.
//!
//! Run with: cargo run --example custom_allocator --features=storage

use anyhow::Result;
#[cfg(feature = "storage")]
use icarus_mcp::storage::allocator::{
    get_pool_stats, get_pooled_buffer, return_pooled_buffer, BufferPool, PooledBuffer,
    TrackingAllocator,
};

#[cfg(feature = "storage")]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("custom_allocator=debug")
        .init();

    println!("💾 Custom Allocator Examples");

    // Example 1: Using the global buffer pool
    println!("\n📦 Example 1: Global Buffer Pool");

    // Get buffers from the pool
    let mut buffer1 = get_pooled_buffer(1024);
    let mut buffer2 = get_pooled_buffer(4096);
    let mut buffer3 = get_pooled_buffer(1024); // Same size as buffer1

    println!("Allocated buffers:");
    println!("  Buffer 1: {} bytes capacity", buffer1.capacity());
    println!("  Buffer 2: {} bytes capacity", buffer2.capacity());
    println!("  Buffer 3: {} bytes capacity", buffer3.capacity());

    // Use the buffers for some work
    buffer1.extend_from_slice(b"Hello from buffer 1!");
    buffer2.extend_from_slice(b"Buffer 2 contains this longer message for testing purposes!");
    buffer3.extend_from_slice(b"Buffer 3 data");

    println!("Buffer contents:");
    println!("  Buffer 1: {}", String::from_utf8_lossy(&buffer1));
    println!("  Buffer 2: {}", String::from_utf8_lossy(&buffer2));
    println!("  Buffer 3: {}", String::from_utf8_lossy(&buffer3));

    // Return buffers to the pool
    return_pooled_buffer(buffer1);
    return_pooled_buffer(buffer2);
    return_pooled_buffer(buffer3);

    // Check pool statistics
    let stats = get_pool_stats();
    println!("Pool statistics after operations:");
    for (size, (total_gets, total_puts, peak_size, current_available)) in &stats {
        println!(
            "  Size {}: {} gets, {} puts, peak {}, {} available",
            size, total_gets, total_puts, peak_size, current_available
        );
    }

    // Example 2: Custom BufferPool
    println!("\n🏗️ Example 2: Custom Buffer Pool");
    let custom_pool = BufferPool::new();

    // Pre-warm the pool with common sizes
    println!("Pre-warming pool with common buffer sizes...");
    let sizes = [256, 512, 1024, 2048, 4096];
    let mut temp_buffers = Vec::new();

    for &size in &sizes {
        for _ in 0..3 {
            temp_buffers.push(custom_pool.get_buffer(size));
        }
    }

    // Return them all to populate the pool
    for buffer in temp_buffers {
        custom_pool.return_buffer(buffer);
    }

    println!("Pool warmed up!");

    // Now use the pre-warmed pool
    let start_time = std::time::Instant::now();
    for _ in 0..1000 {
        let mut buffer = custom_pool.get_buffer(1024);
        buffer.extend_from_slice(b"Performance test data");
        custom_pool.return_buffer(buffer);
    }
    let elapsed = start_time.elapsed();
    println!("1000 allocations/deallocations took: {:?}", elapsed);

    // Example 3: Tracking Allocator
    println!("\n📊 Example 3: Tracking Allocator");
    let _tracking_allocator = TrackingAllocator::new();

    println!("Created tracking allocator wrapping System allocator");
    println!("In a real application, you would configure this as the global allocator");
    println!("Example configuration:");
    println!("  #[global_allocator]");
    println!("  static GLOBAL: TrackingAllocator<System> = TrackingAllocator::new(System);");

    // Example 4: PooledBuffer wrapper
    println!("\n🎯 Example 4: PooledBuffer Wrapper");
    let mut pooled = PooledBuffer::new(2048);

    if let Some(buffer) = pooled.as_mut() {
        buffer.extend_from_slice(
            b"This buffer will be automatically returned to the pool when dropped!",
        );
        println!("PooledBuffer contains: {}", String::from_utf8_lossy(buffer));
        println!("Buffer capacity: {} bytes", buffer.capacity());
    }
    // Buffer is automatically returned to pool when PooledBuffer is dropped

    println!("\n✨ Custom allocator examples completed!");
    println!("💡 Use these patterns in production for better memory efficiency");

    Ok(())
}

#[cfg(not(feature = "storage"))]
fn main() {
    println!("❌ This example requires the 'storage' feature to be enabled.");
    println!("Run with: cargo run --example custom_allocator --features=storage");
}
