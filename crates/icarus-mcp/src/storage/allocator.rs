//! Custom allocators for high-frequency operations
//!
//! Provides optimized memory allocation strategies for performance-critical components

use bytes::BytesMut;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Memory pool for frequently allocated buffer sizes
///
/// Uses pre-allocated memory pools to reduce allocation overhead
/// for common buffer sizes in streaming operations.
pub struct BufferPool {
    pools: HashMap<usize, Mutex<Vec<BytesMut>>>,
    pool_stats: HashMap<usize, PoolStats>,
}

/// Statistics for a memory pool
#[derive(Debug)]
struct PoolStats {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    pool_hits: AtomicUsize,
    pool_misses: AtomicUsize,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            pool_hits: AtomicUsize::new(0),
            pool_misses: AtomicUsize::new(0),
        }
    }
}

impl BufferPool {
    /// Create a new buffer pool with common sizes
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        let mut pool_stats = HashMap::new();

        // Pre-populate pools for common buffer sizes
        let common_sizes = [
            1024,        // Small buffer
            4096,        // Page size
            64 * 1024,   // Default chunk size
            256 * 1024,  // Large buffer
            1024 * 1024, // 1MB buffer
        ];

        for &size in &common_sizes {
            pools.insert(size, Mutex::new(Vec::with_capacity(32)));
            pool_stats.insert(size, PoolStats::default());
        }

        Self { pools, pool_stats }
    }

    /// Get a buffer of the specified size from the pool
    ///
    /// Returns a pre-allocated buffer if available, otherwise allocates new
    pub fn get_buffer(&self, size: usize) -> BytesMut {
        // Find the best fitting pool (smallest size >= requested)
        let pool_size = self.find_best_pool_size(size);

        if let Some(pool_size) = pool_size {
            if let Some(pool) = self.pools.get(&pool_size) {
                if let Ok(mut pool_vec) = pool.try_lock() {
                    if let Some(mut buffer) = pool_vec.pop() {
                        // Pool hit - reuse existing buffer
                        if let Some(stats) = self.pool_stats.get(&pool_size) {
                            stats.pool_hits.fetch_add(1, Ordering::Relaxed);
                            stats.allocations.fetch_add(1, Ordering::Relaxed);
                        }

                        // Resize to requested size if needed
                        buffer.clear();
                        buffer.reserve(size);
                        return buffer;
                    }
                }

                // Pool miss - create new buffer but record the attempt
                if let Some(stats) = self.pool_stats.get(&pool_size) {
                    stats.pool_misses.fetch_add(1, Ordering::Relaxed);
                    stats.allocations.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Fallback: create new buffer
        BytesMut::with_capacity(size)
    }

    /// Return a buffer to the pool for reuse
    ///
    /// Only pools buffers that match common sizes to avoid memory bloat
    pub fn return_buffer(&self, mut buffer: BytesMut) {
        let capacity = buffer.capacity();
        let pool_size = self.find_best_pool_size(capacity);

        if let Some(pool_size) = pool_size {
            // Only pool if capacity is reasonable and matches a pool size
            if capacity <= pool_size * 2 && capacity >= pool_size / 2 {
                if let Some(pool) = self.pools.get(&pool_size) {
                    if let Ok(mut pool_vec) = pool.try_lock() {
                        // Limit pool size to prevent unbounded growth
                        if pool_vec.len() < 64 {
                            buffer.clear();
                            pool_vec.push(buffer);

                            if let Some(stats) = self.pool_stats.get(&pool_size) {
                                stats.deallocations.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        }

        // Buffer not pooled, will be dropped normally
    }

    /// Find the best fitting pool size for a requested size
    fn find_best_pool_size(&self, size: usize) -> Option<usize> {
        self.pools
            .keys()
            .filter(|&&pool_size| pool_size >= size)
            .min()
            .copied()
    }

    /// Get pool statistics for monitoring
    pub fn get_stats(&self) -> HashMap<usize, (usize, usize, usize, usize)> {
        self.pool_stats
            .iter()
            .map(|(&size, stats)| {
                (
                    size,
                    (
                        stats.allocations.load(Ordering::Relaxed),
                        stats.deallocations.load(Ordering::Relaxed),
                        stats.pool_hits.load(Ordering::Relaxed),
                        stats.pool_misses.load(Ordering::Relaxed),
                    ),
                )
            })
            .collect()
    }

    /// Reset pool statistics
    pub fn reset_stats(&self) {
        for stats in self.pool_stats.values() {
            stats.allocations.store(0, Ordering::Relaxed);
            stats.deallocations.store(0, Ordering::Relaxed);
            stats.pool_hits.store(0, Ordering::Relaxed);
            stats.pool_misses.store(0, Ordering::Relaxed);
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local buffer pool for even better performance
thread_local! {
    static BUFFER_POOL: BufferPool = BufferPool::new();
}

/// Get a buffer from the thread-local pool
pub fn get_pooled_buffer(size: usize) -> BytesMut {
    BUFFER_POOL.with(|pool| pool.get_buffer(size))
}

/// Return a buffer to the thread-local pool
pub fn return_pooled_buffer(buffer: BytesMut) {
    BUFFER_POOL.with(|pool| pool.return_buffer(buffer));
}

/// Get statistics from the thread-local pool
pub fn get_pool_stats() -> HashMap<usize, (usize, usize, usize, usize)> {
    BUFFER_POOL.with(|pool| pool.get_stats())
}

/// Custom allocator that tracks allocation patterns
///
/// Wraps the system allocator to provide insights into allocation behavior
/// for optimization purposes.
pub struct TrackingAllocator {
    inner: System,
    stats: AllocationStats,
}

/// Statistics about allocation patterns
#[derive(Debug)]
pub struct AllocationStats {
    /// Total number of allocations performed
    pub total_allocations: AtomicUsize,
    /// Total number of deallocations performed
    pub total_deallocations: AtomicUsize,
    /// Total bytes allocated over lifetime
    pub bytes_allocated: AtomicUsize,
    /// Total bytes deallocated over lifetime
    pub bytes_deallocated: AtomicUsize,
    /// Peak memory usage reached
    pub peak_memory: AtomicUsize,
    /// Current memory usage
    pub current_memory: AtomicUsize,
}

impl Default for AllocationStats {
    fn default() -> Self {
        Self {
            total_allocations: AtomicUsize::new(0),
            total_deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            peak_memory: AtomicUsize::new(0),
            current_memory: AtomicUsize::new(0),
        }
    }
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingAllocator {
    /// Create a new tracking allocator
    pub const fn new() -> Self {
        Self {
            inner: System,
            stats: AllocationStats {
                total_allocations: AtomicUsize::new(0),
                total_deallocations: AtomicUsize::new(0),
                bytes_allocated: AtomicUsize::new(0),
                bytes_deallocated: AtomicUsize::new(0),
                peak_memory: AtomicUsize::new(0),
                current_memory: AtomicUsize::new(0),
            },
        }
    }

    /// Get current allocation statistics
    pub fn stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocations: AtomicUsize::new(
                self.stats.total_allocations.load(Ordering::Relaxed),
            ),
            total_deallocations: AtomicUsize::new(
                self.stats.total_deallocations.load(Ordering::Relaxed),
            ),
            bytes_allocated: AtomicUsize::new(self.stats.bytes_allocated.load(Ordering::Relaxed)),
            bytes_deallocated: AtomicUsize::new(
                self.stats.bytes_deallocated.load(Ordering::Relaxed),
            ),
            peak_memory: AtomicUsize::new(self.stats.peak_memory.load(Ordering::Relaxed)),
            current_memory: AtomicUsize::new(self.stats.current_memory.load(Ordering::Relaxed)),
        }
    }

    /// Reset allocation statistics
    pub fn reset_stats(&self) {
        self.stats.total_allocations.store(0, Ordering::Relaxed);
        self.stats.total_deallocations.store(0, Ordering::Relaxed);
        self.stats.bytes_allocated.store(0, Ordering::Relaxed);
        self.stats.bytes_deallocated.store(0, Ordering::Relaxed);
        self.stats.peak_memory.store(0, Ordering::Relaxed);
        self.stats.current_memory.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);
            self.stats
                .bytes_allocated
                .fetch_add(size, Ordering::Relaxed);

            let current = self.stats.current_memory.fetch_add(size, Ordering::Relaxed) + size;

            // Update peak memory usage
            self.stats.peak_memory.fetch_max(current, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() {
            let size = layout.size();
            self.stats
                .total_deallocations
                .fetch_add(1, Ordering::Relaxed);
            self.stats
                .bytes_deallocated
                .fetch_add(size, Ordering::Relaxed);
            self.stats.current_memory.fetch_sub(size, Ordering::Relaxed);
        }
        self.inner.dealloc(ptr, layout);
    }
}

/// Pooled BytesMut that automatically returns to pool on drop
pub struct PooledBuffer {
    buffer: Option<BytesMut>,
}

impl PooledBuffer {
    /// Create a new pooled buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Some(get_pooled_buffer(capacity)),
        }
    }

    /// Get a mutable reference to the underlying buffer
    pub fn as_mut(&mut self) -> Option<&mut BytesMut> {
        self.buffer.as_mut()
    }

    /// Take the buffer out of the wrapper (won't be returned to pool)
    pub fn take(mut self) -> Option<BytesMut> {
        self.buffer.take()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            return_pooled_buffer(buffer);
        }
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().expect("Buffer was taken")
    }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().expect("Buffer was taken")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = BufferPool::new();
        let stats = pool.get_stats();

        // Should have pools for common sizes
        assert!(stats.contains_key(&1024));
        assert!(stats.contains_key(&4096));
        assert!(stats.contains_key(&(64 * 1024)));
        assert!(stats.contains_key(&(256 * 1024)));
        assert!(stats.contains_key(&(1024 * 1024)));
    }

    #[test]
    fn test_buffer_pool_get_return() {
        let pool = BufferPool::new();

        // Get a buffer
        let buffer1 = pool.get_buffer(1024);
        assert!(buffer1.capacity() >= 1024);

        // Return it
        pool.return_buffer(buffer1);

        // Get another buffer - should reuse the previous one
        let buffer2 = pool.get_buffer(1024);
        assert!(buffer2.capacity() >= 1024);

        // Check stats
        let stats = pool.get_stats();
        if let Some(&(allocations, deallocations, hits, misses)) = stats.get(&1024) {
            assert!(allocations >= 2);
            assert!(deallocations >= 1);
            assert!(hits >= 1 || misses >= 2); // Either hit from pool or miss
        }
    }

    #[test]
    fn test_buffer_pool_size_matching() {
        let pool = BufferPool::new();

        // Request smaller size, should get buffer from 1024 pool
        let buffer = pool.get_buffer(512);
        assert!(buffer.capacity() >= 512);

        // Request exact size
        let buffer = pool.get_buffer(1024);
        assert!(buffer.capacity() >= 1024);

        // Request larger size, should get buffer from 4096 pool
        let buffer = pool.get_buffer(2048);
        assert!(buffer.capacity() >= 2048);
    }

    #[test]
    fn test_tracking_allocator() {
        let allocator = TrackingAllocator::new();

        unsafe {
            let layout = Layout::from_size_align(1024, 8).unwrap();
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());

            let stats = allocator.stats();
            assert_eq!(stats.total_allocations.load(Ordering::Relaxed), 1);
            assert_eq!(stats.bytes_allocated.load(Ordering::Relaxed), 1024);
            assert_eq!(stats.current_memory.load(Ordering::Relaxed), 1024);

            allocator.dealloc(ptr, layout);

            let stats = allocator.stats();
            assert_eq!(stats.total_deallocations.load(Ordering::Relaxed), 1);
            assert_eq!(stats.bytes_deallocated.load(Ordering::Relaxed), 1024);
            assert_eq!(stats.current_memory.load(Ordering::Relaxed), 0);
        }
    }

    #[test]
    fn test_pooled_buffer() {
        let mut pooled = PooledBuffer::new(1024);

        // Should have a buffer with at least the requested capacity
        assert!(pooled.capacity() >= 1024);

        // Should be able to write to it
        pooled.extend_from_slice(b"test data");
        assert_eq!(&pooled[..], b"test data");

        // When dropped, buffer should be returned to pool automatically
    }

    #[test]
    fn test_thread_local_pool() {
        // Get buffer from thread-local pool
        let buffer1 = get_pooled_buffer(1024);
        assert!(buffer1.capacity() >= 1024);

        // Return it
        return_pooled_buffer(buffer1);

        // Get another buffer - should potentially reuse
        let buffer2 = get_pooled_buffer(1024);
        assert!(buffer2.capacity() >= 1024);

        // Get stats
        let stats = get_pool_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_pool_stats_tracking() {
        let pool = BufferPool::new();

        // Perform some operations
        for _ in 0..10 {
            let buffer = pool.get_buffer(1024);
            pool.return_buffer(buffer);
        }

        // Check stats were recorded
        let stats = pool.get_stats();
        if let Some(&(allocations, deallocations, hits, misses)) = stats.get(&1024) {
            assert!(allocations > 0);
            assert!(deallocations > 0);
            assert!(hits > 0 || misses > 0);
        }

        // Reset stats
        pool.reset_stats();
        let stats_after_reset = pool.get_stats();
        for &(allocations, deallocations, hits, misses) in stats_after_reset.values() {
            assert_eq!(allocations, 0);
            assert_eq!(deallocations, 0);
            assert_eq!(hits, 0);
            assert_eq!(misses, 0);
        }
    }
}
