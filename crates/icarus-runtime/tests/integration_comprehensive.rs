//! Comprehensive integration tests for icarus-runtime crate.
//!
//! These tests verify tool execution, registry operations, and caching behavior
//! following the patterns from `rust_best_practices.md`.

use icarus_core::{LegacyToolCall as ToolCall, ToolId};
use icarus_runtime::{
    execute_tool, find_tool, list_tools, ErrorSeverity, ExecutionMetrics, RuntimeError,
    RuntimeResult, ToolExecutor, ToolRegistry,
};
use std::time::Duration;

#[test]
fn test_empty_registry_operations() {
    // Test registry operations when no tools are registered

    // Should not panic with empty registry
    let tools = list_tools();
    assert_eq!(tools.len(), 0);

    // Should return None for any tool lookup
    let tool_id = ToolId::new("nonexistent").unwrap();
    let result = find_tool(&tool_id);
    assert!(result.is_none());

    // Registry validation should pass for empty registry
    let validation_result = ToolRegistry::validate();
    assert!(validation_result.is_ok());

    // Stats should work with empty registry
    let stats = ToolRegistry::stats();
    assert_eq!(stats.tool_count, 0);
    assert!(!stats.has_duplicates);
    assert!(stats.validation_status);
}

#[test]
fn test_registry_index_operations() -> RuntimeResult<()> {
    // Test registry indexing for performance

    let index = ToolRegistry::build_index();
    assert_eq!(index.len(), list_tools().len());

    // Index should be consistent with direct lookup
    let tools = list_tools();
    for tool in &tools {
        if let Ok(tool_id) = ToolId::new(tool.name.as_ref()) {
            let indexed_tool = index.get(&tool_id);
            assert!(indexed_tool.is_some());
            assert_eq!(indexed_tool.unwrap().name, tool.name);
        }
    }

    Ok(())
}

#[test]
fn test_registry_stats_accuracy() {
    let stats = ToolRegistry::stats();

    // Stats should be self-consistent
    assert_eq!(stats.tool_count, list_tools().len());

    // Memory estimation should be reasonable
    if stats.tool_count > 0 {
        assert!(stats.estimated_memory_bytes > 0);
        assert!(stats.estimated_memory_bytes >= stats.tool_count * 100); // At least 100 bytes per tool
        assert!(stats.estimated_memory_bytes <= stats.tool_count * 10000); // At most 10KB per tool
    } else {
        assert_eq!(stats.estimated_memory_bytes, 0);
    }

    // Summary should not be empty
    let summary = stats.summary();
    assert!(!summary.is_empty());
    assert!(summary.contains(&stats.tool_count.to_string()));
}

#[test]
fn test_tool_executor_initialization() {
    // Test various executor initialization patterns

    let default_executor = ToolExecutor::new();
    assert_eq!(default_executor.timeout(), Duration::from_secs(30));
    assert!(!default_executor.cache_enabled());

    let timeout_executor = ToolExecutor::with_timeout(Duration::from_secs(10));
    assert_eq!(timeout_executor.timeout(), Duration::from_secs(10));

    let cached_executor = ToolExecutor::new().with_cache();
    assert!(cached_executor.cache_enabled());

    let default_executor2 = ToolExecutor::default();
    assert_eq!(default_executor2.timeout(), default_executor.timeout());
}

#[test]
fn test_tool_executor_metrics() {
    let executor = ToolExecutor::new();
    let metrics = executor.metrics();

    // Initial metrics should be zero
    assert_eq!(metrics.total_calls, 0);
    assert_eq!(metrics.successful_calls, 0);
    assert_eq!(metrics.failed_calls, 0);
    assert_eq!(metrics.timeouts, 0);
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.success_rate(), 0.0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);

    // Timing metrics should be properly initialized
    assert_eq!(metrics.avg_execution_time_ms, 0.0);
    assert_eq!(metrics.min_execution_time_ms, f64::INFINITY);
    assert_eq!(metrics.max_execution_time_ms, 0.0);
}

#[test]
fn test_tool_executor_cache_key_generation() {
    let executor = ToolExecutor::new();
    let tool_call =
        ToolCall::new(ToolId::new("test_tool").unwrap()).with_arguments(r#"{"arg": "value"}"#);

    let key1 = executor.generate_cache_key(&tool_call);
    let key2 = executor.generate_cache_key(&tool_call);

    // Same tool call should generate same cache key
    assert_eq!(key1, key2);

    // Different tool calls should generate different keys
    let different_call =
        ToolCall::new(ToolId::new("other_tool").unwrap()).with_arguments(r#"{"arg": "value"}"#);
    let key3 = executor.generate_cache_key(&different_call);
    assert_ne!(key1, key3);

    // Different arguments should generate different keys
    let different_args_call = ToolCall::new(ToolId::new("test_tool").unwrap())
        .with_arguments(r#"{"arg": "different_value"}"#);
    let key4 = executor.generate_cache_key(&different_args_call);
    assert_ne!(key1, key4);
}

#[test]
fn test_tool_executor_cache_operations() {
    let mut executor = ToolExecutor::new().with_cache();

    // Cache should start empty
    let initial_metrics = executor.metrics();
    assert_eq!(initial_metrics.cache_hits, 0);

    // Clear cache should not panic
    executor.clear_cache();

    // Reset metrics should work
    executor.reset_metrics();
    let reset_metrics = executor.metrics();
    assert_eq!(reset_metrics.total_calls, 0);
}

#[test]
fn test_error_handling_completeness() {
    // Test all error types and their properties

    let errors = vec![
        RuntimeError::tool_not_found("missing_tool"),
        RuntimeError::execution_failed("failing_tool", "timeout"),
        RuntimeError::invalid_arguments("tool_with_bad_args", "missing required field"),
        RuntimeError::registry_error("corrupted registry"),
    ];

    for error in errors {
        // All errors should have non-empty messages
        assert!(!error.to_string().is_empty());

        // User messages should be appropriate
        let user_message = error.user_message();
        assert!(!user_message.is_empty());

        // Error severity should be appropriate
        let severity = error.severity();
        assert!(severity.level() <= 3); // Should be valid severity level

        // Severity string representation should work
        assert!(!severity.as_str().is_empty());
    }
}

#[test]
fn test_error_severity_ordering() {
    // ErrorSeverity is already imported at the top

    // Test severity ordering
    assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
    assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
    assert!(ErrorSeverity::Warning > ErrorSeverity::Info);

    // Test level consistency
    assert!(ErrorSeverity::Critical.level() > ErrorSeverity::Error.level());
    assert!(ErrorSeverity::Error.level() > ErrorSeverity::Warning.level());
    assert!(ErrorSeverity::Warning.level() > ErrorSeverity::Info.level());
}

#[test]
fn test_tool_execution_mock_behavior() -> RuntimeResult<()> {
    // Test the mock execution behavior (since we don't have real tools registered)

    let tool_call =
        ToolCall::new(ToolId::new("mock_tool").unwrap()).with_arguments(r#"{"test": "data"}"#);

    #[cfg(not(feature = "async"))]
    {
        let mut executor = ToolExecutor::new();

        // Execution should fail for non-existent tools
        let result = executor.execute(tool_call);
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::ToolNotFound { tool_id } => {
                assert_eq!(tool_id, "mock_tool");
            }
            other => panic!("Expected ToolNotFound error, got: {:?}", other),
        }
    }

    #[cfg(feature = "async")]
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut executor = ToolExecutor::new();

        let result = rt.block_on(async { executor.execute(tool_call).await });
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::ToolNotFound { tool_id } => {
                assert_eq!(tool_id, "mock_tool");
            }
            other => panic!("Expected ToolNotFound error, got: {:?}", other),
        }
    }

    Ok(())
}

#[test]
fn test_convenience_functions() -> RuntimeResult<()> {
    // Test convenience functions

    let tool_call = ToolCall::new(ToolId::new("convenience_test").unwrap());

    #[cfg(not(feature = "async"))]
    {
        let result = execute_tool(tool_call);
        assert!(result.is_err()); // Should fail for non-existent tool
    }

    #[cfg(feature = "async")]
    {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async { execute_tool(tool_call).await });
        assert!(result.is_err()); // Should fail for non-existent tool
    }

    Ok(())
}

#[test]
fn test_concurrent_registry_access() -> RuntimeResult<()> {
    use std::thread;

    // Test that registry operations are safe for concurrent access
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                // Each thread performs registry operations
                let tools = list_tools();
                let stats = ToolRegistry::stats();
                let validation = ToolRegistry::validate();

                // Operations should complete successfully
                assert!(validation.is_ok());
                assert_eq!(tools.len(), stats.tool_count);

                // Try to find a tool that doesn't exist
                let tool_id = ToolId::new(format!("thread_tool_{i}")).unwrap();
                let result = find_tool(&tool_id);
                assert!(result.is_none());
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

#[test]
fn test_registry_edge_cases() -> RuntimeResult<()> {
    // Test registry behavior with edge cases

    // Very long tool name
    let long_name = "a".repeat(1000);
    let result = ToolId::new(&long_name);
    assert!(result.is_err()); // Should be rejected

    // Empty tool name
    let empty_name = "";
    let result = ToolId::new(empty_name);
    assert!(result.is_err()); // Should be rejected

    // Special characters in tool name
    let special_chars = "tool@domain.com";
    let result = ToolId::new(special_chars);
    assert!(result.is_err()); // Should be rejected

    Ok(())
}

#[test]
fn test_tool_executor_timeout_behavior() {
    // Test timeout configuration and validation

    let short_timeout = Duration::from_millis(1);
    let executor = ToolExecutor::with_timeout(short_timeout);
    assert_eq!(executor.timeout(), short_timeout);

    let long_timeout = Duration::from_secs(3600); // 1 hour
    let executor = ToolExecutor::with_timeout(long_timeout);
    assert_eq!(executor.timeout(), long_timeout);

    let zero_timeout = Duration::from_millis(0);
    let executor = ToolExecutor::with_timeout(zero_timeout);
    assert_eq!(executor.timeout(), zero_timeout);
}

#[test]
fn test_metrics_calculation_accuracy() {
    // ExecutionMetrics is already imported at the top

    let mut metrics = ExecutionMetrics::new();

    // Test initial state
    assert_eq!(metrics.success_rate(), 0.0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);

    // Simulate some metrics updates
    metrics.total_calls = 10;
    metrics.successful_calls = 8;
    metrics.cache_hits = 3;

    assert_eq!(metrics.success_rate(), 80.0);
    assert_eq!(metrics.cache_hit_rate(), 30.0);

    // Test edge cases
    metrics.total_calls = 0;
    assert_eq!(metrics.success_rate(), 0.0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);

    // Test 100% success rate
    metrics.total_calls = 5;
    metrics.successful_calls = 5;
    metrics.cache_hits = 0;
    assert_eq!(metrics.success_rate(), 100.0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);
}

#[test]
fn test_error_conversion_chain() {
    use icarus_core::IcarusError;

    // Test conversion from IcarusError to RuntimeError
    let core_error = IcarusError::internal_error("Core error message");
    let runtime_error: RuntimeError = core_error.into();

    match runtime_error {
        RuntimeError::CoreError { source } => {
            assert!(source.to_string().contains("Core error message"));
        }
        other => panic!("Expected CoreError, got: {:?}", other),
    }

    // Test JSON error conversion
    let json_error = serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "JSON test error",
    ));
    let runtime_error = RuntimeError::json_error("test_tool", json_error);

    match runtime_error {
        RuntimeError::JsonError { tool_id, source } => {
            assert_eq!(tool_id, "test_tool");
            assert!(source.to_string().contains("JSON test error"));
        }
        other => panic!("Expected JsonError, got: {:?}", other),
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_executor_behavior() -> RuntimeResult<()> {
    // Test async-specific executor behavior

    let mut executor = ToolExecutor::new().with_cache();
    let tool_call =
        ToolCall::new(ToolId::new("async_test_tool").unwrap()).with_arguments(r#"{"async": true}"#);

    // Should fail for non-existent tool
    let result = executor.execute(tool_call).await;
    assert!(result.is_err());

    // Metrics should be updated even for failures
    let metrics = executor.metrics();
    assert_eq!(metrics.total_calls, 1);
    assert_eq!(metrics.successful_calls, 0);

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_error_types() {
    // Test async-specific error types

    let async_error = RuntimeError::async_error("Async operation failed");
    assert!(async_error.to_string().contains("Async operation failed"));

    let user_message = async_error.user_message();
    assert!(!user_message.is_empty());
    assert_eq!(async_error.severity(), ErrorSeverity::Error);
}

#[test]
fn test_memory_safety_and_cleanup() {
    // Test that resources are properly cleaned up

    let mut executor = ToolExecutor::new().with_cache();

    // Fill cache with some data
    for i in 0..10 {
        let tool_call = ToolCall::new(ToolId::new(format!("tool_{i}")).unwrap());
        let _key = executor.generate_cache_key(&tool_call);
    }

    // Clear cache should free memory
    executor.clear_cache();

    // Reset metrics should restore initial state
    executor.reset_metrics();
    let metrics = executor.metrics();
    assert_eq!(metrics.total_calls, 0);
    assert_eq!(metrics.cache_hits, 0);
}

#[test]
fn test_registry_validation_edge_cases() {
    // Test registry validation with various scenarios

    // Normal validation should pass
    let result = ToolRegistry::validate();
    assert!(result.is_ok());

    // Empty registry should be valid
    let stats = ToolRegistry::stats();
    if stats.tool_count == 0 {
        assert!(stats.validation_status);
    }
}
