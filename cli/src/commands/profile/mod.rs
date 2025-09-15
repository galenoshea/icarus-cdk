//! Performance profiling and benchmarking commands

pub mod analyze;
pub mod bench;
pub mod canister;

/// Common profiling utilities and types
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Performance measurement results
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub duration: Duration,
    pub requests_per_second: f64,
    pub average_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub p95_response_time: Duration,
    pub error_rate: f64,
    pub throughput_mb_per_sec: f64,
}

/// Benchmark result for a single operation
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub metrics: PerformanceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Collection of benchmark results
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub results: Vec<BenchmarkResult>,
    pub summary: PerformanceMetrics,
    pub environment: EnvironmentInfo,
}

/// System environment information
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub rust_version: String,
    pub icarus_version: String,
}

impl EnvironmentInfo {
    /// Collect current environment information
    pub fn collect() -> Self {
        let info = os_info::get();
        Self {
            os: format!("{} {}", info.os_type(), info.version()),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus::get(),
            total_memory: sysinfo::System::new_all().total_memory(),
            rust_version: rustc_version::version()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            icarus_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Utility functions for performance measurement
pub mod utils {
    use super::*;

    /// Calculate performance metrics from a collection of response times
    pub fn calculate_metrics(
        response_times: &[Duration],
        errors: usize,
        throughput_bytes: u64,
    ) -> PerformanceMetrics {
        if response_times.is_empty() {
            return PerformanceMetrics {
                duration: Duration::from_secs(0),
                requests_per_second: 0.0,
                average_response_time: Duration::from_secs(0),
                min_response_time: Duration::from_secs(0),
                max_response_time: Duration::from_secs(0),
                p95_response_time: Duration::from_secs(0),
                error_rate: 0.0,
                throughput_mb_per_sec: 0.0,
            };
        }

        let total_requests = response_times.len() + errors;
        let total_duration: Duration = response_times.iter().sum();
        let average_response_time = total_duration / response_times.len() as u32;

        let mut sorted_times = response_times.to_vec();
        sorted_times.sort();

        let min_response_time = *sorted_times.first().unwrap();
        let max_response_time = *sorted_times.last().unwrap();
        let p95_index = (sorted_times.len() as f64 * 0.95).floor() as usize;
        let p95_response_time = sorted_times
            .get(p95_index)
            .copied()
            .unwrap_or(max_response_time);

        let requests_per_second = response_times.len() as f64 / total_duration.as_secs_f64();
        let error_rate = errors as f64 / total_requests as f64;
        let throughput_mb_per_sec =
            throughput_bytes as f64 / (1024.0 * 1024.0) / total_duration.as_secs_f64();

        PerformanceMetrics {
            duration: total_duration,
            requests_per_second,
            average_response_time,
            min_response_time,
            max_response_time,
            p95_response_time,
            error_rate,
            throughput_mb_per_sec,
        }
    }

    /// Format duration for human readability
    pub fn format_duration(duration: Duration) -> String {
        let nanos = duration.as_nanos();
        if nanos < 1_000 {
            format!("{}ns", nanos)
        } else if nanos < 1_000_000 {
            format!("{:.2}μs", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.2}ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
        }
    }

    /// Format bytes for human readability
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
