//! Enhanced parallel test execution support

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Test execution metrics for performance monitoring
pub struct TestMetrics {
    start_time: Instant,
    test_name: String,
}

impl TestMetrics {
    pub fn new(test_name: &str) -> Self {
        println!("🚀 Starting test: {}", test_name);
        Self {
            start_time: Instant::now(),
            test_name: test_name.to_string(),
        }
    }
}

impl Drop for TestMetrics {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        println!("✅ Test {} completed in {:?}", self.test_name, duration);
    }
}

/// Test cache type for cleaner code
type TestCache = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// Global test result cache to avoid re-running expensive operations
static TEST_CACHE: Lazy<TestCache> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Cache test results to speed up subsequent runs
pub fn cache_test_result(key: &str, data: Vec<u8>) {
    if let Ok(mut cache) = TEST_CACHE.lock() {
        cache.insert(key.to_string(), data);
    }
}

/// Retrieve cached test results
pub fn get_cached_result(key: &str) -> Option<Vec<u8>> {
    if let Ok(cache) = TEST_CACHE.lock() {
        cache.get(key).cloned()
    } else {
        None
    }
}

/// Get a unique test project directory based on test name or matrix
pub fn get_test_project_dir(test_name: &str) -> PathBuf {
    // Check for environment variable set by CI matrix
    if let Ok(base_dir) = env::var("ICARUS_TEST_PROJECT_DIR") {
        // Append test_name to make it unique even in CI
        PathBuf::from(base_dir).join(test_name)
    } else {
        // Use a unique directory per test for local parallel execution
        env::temp_dir().join(format!("icarus-e2e-{}", test_name))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_tracking() {
        let _metrics = TestMetrics::new("test_metrics");
        // Metrics will be printed when dropped
    }

    #[test]
    fn test_cache_operations() {
        cache_test_result("test_key", vec![1, 2, 3]);
        assert_eq!(get_cached_result("test_key"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_unique_project_dirs() {
        let dir1 = get_test_project_dir("test1");
        let dir2 = get_test_project_dir("test2");
        assert_ne!(dir1, dir2);
    }
}
