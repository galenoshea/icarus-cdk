//! Profile-guided optimization utilities
//!
//! Provides performance profiling, analysis, and optimization recommendations

#![allow(unsafe_code)] // Thread-local static lifetime requires unsafe transmute

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Performance metrics collector
#[derive(Debug)]
pub struct PerformanceProfiler {
    /// Function timing measurements
    function_timings: Arc<Mutex<HashMap<String, FunctionStats>>>,
    /// Memory allocation tracking
    allocation_stats: Arc<Mutex<HashMap<String, AllocationStats>>>,
    /// Hot path detection
    hot_paths: Arc<Mutex<HashMap<String, HotPathStats>>>,
    /// Performance recommendations
    recommendations: Arc<Mutex<Vec<OptimizationHint>>>,
}

/// Statistics for function performance
#[derive(Debug)]
struct FunctionStats {
    call_count: AtomicUsize,
    total_time: AtomicU64, // nanoseconds
    min_time: AtomicU64,
    max_time: AtomicU64,
    avg_time: AtomicU64,
}

impl Default for FunctionStats {
    fn default() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            total_time: AtomicU64::new(0),
            min_time: AtomicU64::new(u64::MAX),
            max_time: AtomicU64::new(0),
            avg_time: AtomicU64::new(0),
        }
    }
}

impl FunctionStats {
    fn update(&self, duration: Duration) {
        let nanos = duration.as_nanos() as u64;
        let count = self.call_count.fetch_add(1, Ordering::Relaxed);
        let total = self.total_time.fetch_add(nanos, Ordering::Relaxed);

        // Update min time
        let mut current_min = self.min_time.load(Ordering::Relaxed);
        while current_min > nanos {
            match self.min_time.compare_exchange_weak(
                current_min,
                nanos,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // Update max time
        let mut current_max = self.max_time.load(Ordering::Relaxed);
        while current_max < nanos {
            match self.max_time.compare_exchange_weak(
                current_max,
                nanos,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // Update average
        let new_avg = (total + nanos) / (count + 1) as u64;
        self.avg_time.store(new_avg, Ordering::Relaxed);
    }
}

/// Memory allocation statistics
#[derive(Debug)]
struct AllocationStats {
    allocation_count: AtomicUsize,
    total_bytes: AtomicUsize,
    avg_size: AtomicUsize,
    peak_single_allocation: AtomicUsize,
}

impl Default for AllocationStats {
    fn default() -> Self {
        Self {
            allocation_count: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            avg_size: AtomicUsize::new(0),
            peak_single_allocation: AtomicUsize::new(0),
        }
    }
}

impl AllocationStats {
    fn record_allocation(&self, size: usize) {
        let count = self.allocation_count.fetch_add(1, Ordering::Relaxed);
        let total = self.total_bytes.fetch_add(size, Ordering::Relaxed);
        let new_avg = (total + size) / (count + 1);
        self.avg_size.store(new_avg, Ordering::Relaxed);

        // Update peak allocation size
        let mut current_peak = self.peak_single_allocation.load(Ordering::Relaxed);
        while current_peak < size {
            match self.peak_single_allocation.compare_exchange_weak(
                current_peak,
                size,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_peak = x,
            }
        }
    }
}

/// Hot path detection statistics
#[derive(Debug)]
struct HotPathStats {
    execution_count: AtomicUsize,
    total_time: AtomicU64,
}

impl Default for HotPathStats {
    fn default() -> Self {
        Self {
            execution_count: AtomicUsize::new(0),
            total_time: AtomicU64::new(0),
        }
    }
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    /// Target function or code path
    pub target: String,
    /// Type of optimization suggested
    pub optimization_type: OptimizationType,
    /// Expected performance impact
    pub impact: PerformanceImpact,
    /// Detailed recommendation
    pub description: String,
    /// Confidence score (0-100)
    pub confidence: u8,
}

/// Types of optimizations that can be suggested
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationType {
    /// Use memory pooling for frequent allocations
    MemoryPooling,
    /// Enable SIMD optimizations
    SimdOptimization,
    /// Use zero-copy serialization
    ZeroCopySerialization,
    /// Cache frequently computed values
    Caching,
    /// Batch similar operations
    Batching,
    /// Use more efficient data structures
    DataStructureOptimization,
    /// Reduce function call overhead
    Inlining,
    /// Parallel processing opportunities
    Parallelization,
}

/// Expected performance impact
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceImpact {
    /// Minor improvement (< 10%)
    Minor,
    /// Moderate improvement (10-30%)
    Moderate,
    /// Significant improvement (30-50%)
    Significant,
    /// Major improvement (> 50%)
    Major,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            function_timings: Arc::new(Mutex::new(HashMap::new())),
            allocation_stats: Arc::new(Mutex::new(HashMap::new())),
            hot_paths: Arc::new(Mutex::new(HashMap::new())),
            recommendations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start timing a function
    pub fn start_timer(&self, function_name: &str) -> TimingGuard {
        TimingGuard::new(function_name.to_string(), self.function_timings.clone())
    }

    /// Record memory allocation
    pub fn record_allocation(&self, context: &str, size: usize) {
        if let Ok(mut stats) = self.allocation_stats.lock() {
            let entry = stats.entry(context.to_string()).or_default();
            entry.record_allocation(size);
        }
    }

    /// Mark a hot path execution
    pub fn mark_hot_path(&self, path_name: &str, duration: Duration) {
        if let Ok(mut hot_paths) = self.hot_paths.lock() {
            let entry = hot_paths.entry(path_name.to_string()).or_default();
            entry.execution_count.fetch_add(1, Ordering::Relaxed);
            entry
                .total_time
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    /// Generate optimization recommendations based on collected data
    pub fn generate_recommendations(&self) -> Vec<OptimizationHint> {
        let mut recommendations = Vec::new();

        // Analyze function timings for slow functions
        if let Ok(timings) = self.function_timings.lock() {
            for (function_name, stats) in timings.iter() {
                let avg_time = stats.avg_time.load(Ordering::Relaxed);
                let call_count = stats.call_count.load(Ordering::Relaxed);

                // Suggest optimizations for slow or frequently called functions
                if avg_time > 100_000 || call_count > 1 {
                    // > 0.1ms avg or > 1 calls
                    if function_name.contains("serialize") || function_name.contains("deserialize")
                    {
                        recommendations.push(OptimizationHint {
                            target: function_name.clone(),
                            optimization_type: OptimizationType::ZeroCopySerialization,
                            impact: if avg_time > 10_000_000 {
                                PerformanceImpact::Significant
                            } else {
                                PerformanceImpact::Moderate
                            },
                            description: "Consider using zero-copy serialization to reduce allocation overhead".to_string(),
                            confidence: 80,
                        });
                    }

                    if function_name.contains("copy") || function_name.contains("process") {
                        recommendations.push(OptimizationHint {
                            target: function_name.clone(),
                            optimization_type: OptimizationType::SimdOptimization,
                            impact: PerformanceImpact::Moderate,
                            description: "SIMD optimizations could accelerate data processing"
                                .to_string(),
                            confidence: 70,
                        });
                    }
                }
            }
        }

        // Analyze allocation patterns
        if let Ok(allocations) = self.allocation_stats.lock() {
            for (context, stats) in allocations.iter() {
                let count = stats.allocation_count.load(Ordering::Relaxed);
                let avg_size = stats.avg_size.load(Ordering::Relaxed);

                if count > 10 && avg_size < 1024 * 1024 {
                    // Many small allocations
                    recommendations.push(OptimizationHint {
                        target: context.clone(),
                        optimization_type: OptimizationType::MemoryPooling,
                        impact: PerformanceImpact::Moderate,
                        description: format!(
                            "Frequent allocations detected ({} allocs, avg {} bytes). Memory pooling recommended.",
                            count, avg_size
                        ),
                        confidence: 85,
                    });
                }
            }
        }

        // Store recommendations
        if let Ok(mut stored_recs) = self.recommendations.lock() {
            stored_recs.extend(recommendations.clone());
        }

        recommendations
    }

    /// Get current performance statistics
    pub fn get_statistics(&self) -> PerformanceReport {
        let mut function_stats = HashMap::new();
        let mut allocation_summary = HashMap::new();
        let mut hot_path_summary = HashMap::new();

        if let Ok(timings) = self.function_timings.lock() {
            for (name, stats) in timings.iter() {
                function_stats.insert(
                    name.clone(),
                    FunctionReport {
                        call_count: stats.call_count.load(Ordering::Relaxed),
                        avg_time_nanos: stats.avg_time.load(Ordering::Relaxed),
                        min_time_nanos: stats.min_time.load(Ordering::Relaxed),
                        max_time_nanos: stats.max_time.load(Ordering::Relaxed),
                        total_time_nanos: stats.total_time.load(Ordering::Relaxed),
                    },
                );
            }
        }

        if let Ok(allocations) = self.allocation_stats.lock() {
            for (name, stats) in allocations.iter() {
                allocation_summary.insert(
                    name.clone(),
                    AllocationReport {
                        allocation_count: stats.allocation_count.load(Ordering::Relaxed),
                        total_bytes: stats.total_bytes.load(Ordering::Relaxed),
                        avg_size: stats.avg_size.load(Ordering::Relaxed),
                        peak_single_allocation: stats
                            .peak_single_allocation
                            .load(Ordering::Relaxed),
                    },
                );
            }
        }

        if let Ok(hot_paths) = self.hot_paths.lock() {
            for (name, stats) in hot_paths.iter() {
                hot_path_summary.insert(
                    name.clone(),
                    HotPathReport {
                        execution_count: stats.execution_count.load(Ordering::Relaxed),
                        total_time_nanos: stats.total_time.load(Ordering::Relaxed),
                    },
                );
            }
        }

        PerformanceReport {
            function_stats,
            allocation_summary,
            hot_path_summary,
        }
    }

    /// Reset all collected statistics
    pub fn reset_statistics(&self) {
        if let Ok(mut timings) = self.function_timings.lock() {
            timings.clear();
        }
        if let Ok(mut allocations) = self.allocation_stats.lock() {
            allocations.clear();
        }
        if let Ok(mut hot_paths) = self.hot_paths.lock() {
            hot_paths.clear();
        }
        if let Ok(mut recommendations) = self.recommendations.lock() {
            recommendations.clear();
        }
    }
}

/// RAII timing guard that automatically records function execution time
pub struct TimingGuard {
    function_name: String,
    start_time: Instant,
    function_timings: Arc<Mutex<HashMap<String, FunctionStats>>>,
}

impl TimingGuard {
    fn new(
        function_name: String,
        function_timings: Arc<Mutex<HashMap<String, FunctionStats>>>,
    ) -> Self {
        Self {
            function_name,
            start_time: Instant::now(),
            function_timings,
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();

        if let Ok(mut timings) = self.function_timings.lock() {
            let entry = timings.entry(self.function_name.clone()).or_default();
            entry.update(duration);
        }
    }
}

/// Performance report containing all collected statistics
#[derive(Debug)]
pub struct PerformanceReport {
    /// Function timing statistics
    pub function_stats: HashMap<String, FunctionReport>,
    /// Memory allocation statistics
    pub allocation_summary: HashMap<String, AllocationReport>,
    /// Hot path execution statistics
    pub hot_path_summary: HashMap<String, HotPathReport>,
}

/// Function performance report
#[derive(Debug)]
pub struct FunctionReport {
    /// Number of times function was called
    pub call_count: usize,
    /// Average execution time in nanoseconds
    pub avg_time_nanos: u64,
    /// Minimum execution time in nanoseconds
    pub min_time_nanos: u64,
    /// Maximum execution time in nanoseconds
    pub max_time_nanos: u64,
    /// Total execution time in nanoseconds
    pub total_time_nanos: u64,
}

/// Memory allocation report
#[derive(Debug)]
pub struct AllocationReport {
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub avg_size: usize,
    /// Largest single allocation
    pub peak_single_allocation: usize,
}

/// Hot path execution report
#[derive(Debug)]
pub struct HotPathReport {
    /// Number of executions
    pub execution_count: usize,
    /// Total time spent in this path
    pub total_time_nanos: u64,
}

/// Convenience macro for timing function execution
#[macro_export]
macro_rules! profile_function {
    ($profiler:expr, $func_name:expr, $code:block) => {{
        let _guard = $profiler.start_timer($func_name);
        $code
    }};
}

// Thread-local profiler for low-overhead profiling
thread_local! {
    static THREAD_PROFILER: PerformanceProfiler = PerformanceProfiler::new();
}

/// Get the thread-local profiler
pub fn thread_profiler() -> &'static PerformanceProfiler {
    // Note: This is a simplified approach. In practice, you might want to use
    // a more sophisticated thread-local storage mechanism.
    THREAD_PROFILER.with(|p| unsafe {
        // SAFETY: This is safe because thread_local ensures each thread has its own copy
        std::mem::transmute::<&PerformanceProfiler, &'static PerformanceProfiler>(p)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        let stats = profiler.get_statistics();

        assert!(stats.function_stats.is_empty());
        assert!(stats.allocation_summary.is_empty());
        assert!(stats.hot_path_summary.is_empty());
    }

    #[test]
    fn test_function_timing() {
        let profiler = PerformanceProfiler::new();

        {
            let _guard = profiler.start_timer("test_function");
            thread::sleep(std::time::Duration::from_millis(1));
        }

        let stats = profiler.get_statistics();
        assert!(stats.function_stats.contains_key("test_function"));

        let func_stats = &stats.function_stats["test_function"];
        assert_eq!(func_stats.call_count, 1);
        assert!(func_stats.avg_time_nanos > 0);
    }

    #[test]
    fn test_allocation_recording() {
        let profiler = PerformanceProfiler::new();

        profiler.record_allocation("buffer_creation", 1024);
        profiler.record_allocation("buffer_creation", 2048);

        let stats = profiler.get_statistics();
        assert!(stats.allocation_summary.contains_key("buffer_creation"));

        let alloc_stats = &stats.allocation_summary["buffer_creation"];
        assert_eq!(alloc_stats.allocation_count, 2);
        assert_eq!(alloc_stats.total_bytes, 3072);
        assert_eq!(alloc_stats.peak_single_allocation, 2048);
    }

    #[test]
    fn test_hot_path_tracking() {
        let profiler = PerformanceProfiler::new();

        profiler.mark_hot_path("critical_loop", Duration::from_millis(5));
        profiler.mark_hot_path("critical_loop", Duration::from_millis(3));

        let stats = profiler.get_statistics();
        assert!(stats.hot_path_summary.contains_key("critical_loop"));

        let hot_path = &stats.hot_path_summary["critical_loop"];
        assert_eq!(hot_path.execution_count, 2);
        assert!(hot_path.total_time_nanos > 0);
    }

    #[test]
    fn test_optimization_recommendations() {
        let profiler = PerformanceProfiler::new();

        // Simulate a slow serialization function
        {
            let _guard = profiler.start_timer("serialize_data");
            thread::sleep(Duration::from_millis(2)); // Simulate slow operation
        }

        // Record multiple calls to trigger recommendation
        for _ in 0..5 {
            let _guard = profiler.start_timer("serialize_data");
        }

        profiler.record_allocation("frequent_allocs", 512);
        for _ in 0..200 {
            // Many small allocations
            profiler.record_allocation("frequent_allocs", 256);
        }

        let recommendations = profiler.generate_recommendations();
        assert!(!recommendations.is_empty());

        // Should recommend zero-copy serialization and memory pooling
        let has_serialization_rec = recommendations
            .iter()
            .any(|r| r.optimization_type == OptimizationType::ZeroCopySerialization);
        let has_pooling_rec = recommendations
            .iter()
            .any(|r| r.optimization_type == OptimizationType::MemoryPooling);

        assert!(has_serialization_rec);
        assert!(has_pooling_rec);
    }

    #[test]
    fn test_statistics_reset() {
        let profiler = PerformanceProfiler::new();

        {
            let _guard = profiler.start_timer("test_function");
        }

        profiler.record_allocation("test_context", 1024);

        let stats_before = profiler.get_statistics();
        assert!(!stats_before.function_stats.is_empty());
        assert!(!stats_before.allocation_summary.is_empty());

        profiler.reset_statistics();

        let stats_after = profiler.get_statistics();
        assert!(stats_after.function_stats.is_empty());
        assert!(stats_after.allocation_summary.is_empty());
    }

    #[test]
    fn test_profile_function_macro() {
        let profiler = PerformanceProfiler::new();

        let result = profile_function!(profiler, "macro_test", {
            thread::sleep(Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);

        let stats = profiler.get_statistics();
        assert!(stats.function_stats.contains_key("macro_test"));
    }
}
