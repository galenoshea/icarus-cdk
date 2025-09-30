//! Performance benchmarks for icarus-macros crate.
//!
//! These benchmarks measure macro expansion performance and generated code efficiency
//! following the patterns from `rust_best_practices.md`.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use icarus_macros::tool;
use std::time::Duration;

/// Benchmark simple tool function performance
fn bench_simple_tool_performance(c: &mut Criterion) {
    #[tool]
    fn simple_add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[tool]
    fn simple_string_op(s: String) -> String {
        s.to_uppercase()
    }

    let mut group = c.benchmark_group("simple_tools");

    group.bench_function("simple_add", |b| {
        b.iter(|| simple_add(42, 58));
    });

    group.bench_function("simple_string_op", |b| {
        b.iter(|| simple_string_op("hello world".to_string()));
    });

    group.finish();
}

/// Benchmark tool functions with complex parameters
fn bench_complex_parameter_tools(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Clone)]
    struct ComplexData {
        id: u64,
        name: String,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
        score: f64,
    }

    #[tool]
    fn process_complex_data(data: ComplexData) -> String {
        format!(
            "Processed {} (id: {}, tags: {}, score: {:.2})",
            data.name,
            data.id,
            data.tags.len(),
            data.score
        )
    }

    let test_data = ComplexData {
        id: 12345,
        name: "Test Item".to_string(),
        tags: vec![
            "important".to_string(),
            "urgent".to_string(),
            "review".to_string(),
        ],
        metadata: {
            let mut map = HashMap::new();
            map.insert("source".to_string(), "api".to_string());
            map.insert("version".to_string(), "1.0".to_string());
            map.insert("category".to_string(), "test".to_string());
            map
        },
        score: 95.7,
    };

    c.bench_function("complex_parameter_tool", |b| {
        b.iter(|| process_complex_data(test_data.clone()));
    });
}

/// Benchmark async tool performance
fn bench_async_tool_performance(c: &mut Criterion) {
    use tokio::runtime::Runtime;

    #[tool]
    async fn async_computation(n: u32) -> u64 {
        let mut result = 0u64;
        for i in 0..n {
            result += u64::from(i);
            // Simulate async work
            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }
        result
    }

    #[tool]
    async fn async_string_processing(input: String) -> String {
        tokio::task::yield_now().await;
        input.chars().rev().collect()
    }

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async_tools");

    for size in &[100u32, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("async_computation", size),
            size,
            |b, &size| {
                b.iter(|| rt.block_on(async_computation(size)));
            },
        );
    }

    group.bench_function("async_string_processing", |b| {
        b.iter(|| {
            rt.block_on(async_string_processing(
                "hello world from async".to_string(),
            ))
        });
    });

    group.finish();
}

/// Benchmark tool functions with many parameters
fn bench_many_parameters_tool(c: &mut Criterion) {
    #[tool]
    fn many_params_tool(
        p1: i32,
        p2: i32,
        p3: i32,
        p4: i32,
        p5: i32,
        p6: String,
        p7: String,
        p8: String,
        p9: String,
        p10: String,
        p11: bool,
        p12: bool,
        p13: bool,
        p14: bool,
        p15: bool,
        p16: f64,
        p17: f64,
        p18: f64,
        p19: f64,
        p20: f64,
    ) -> String {
        format!(
            "ints: {} {} {} {} {}, strings: {} {} {} {} {}, bools: {} {} {} {} {}, floats: {:.1} {:.1} {:.1} {:.1} {:.1}",
            p1, p2, p3, p4, p5,
            p6, p7, p8, p9, p10,
            p11, p12, p13, p14, p15,
            p16, p17, p18, p19, p20
        )
    }

    c.bench_function("many_parameters_tool", |b| {
        b.iter(|| {
            many_params_tool(
                1,
                2,
                3,
                4,
                5,
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                true,
                false,
                true,
                false,
                true,
                1.1,
                2.2,
                3.3,
                4.4,
                5.5,
            )
        });
    });
}

/// Benchmark JSON serialization/deserialization performance
fn bench_json_performance(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct JsonTestParams {
        id: u64,
        name: String,
        values: Vec<i32>,
        enabled: bool,
        config: std::collections::HashMap<String, String>,
    }

    let test_params = JsonTestParams {
        id: 123456789,
        name: "Test Object".to_string(),
        values: (0..100).collect(),
        enabled: true,
        config: {
            let mut map = std::collections::HashMap::new();
            for i in 0..20 {
                map.insert(format!("key_{}", i), format!("value_{}", i));
            }
            map
        },
    };

    let json_string = serde_json::to_string(&test_params).unwrap();

    let mut group = c.benchmark_group("json_operations");

    group.bench_function("serialize_to_json", |b| {
        b.iter(|| serde_json::to_string(&test_params));
    });

    group.bench_function("deserialize_from_json", |b| {
        b.iter(|| serde_json::from_str::<JsonTestParams>(&json_string));
    });

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling_performance(c: &mut Criterion) {
    #[tool]
    fn error_prone_tool(should_error: bool, input: String) -> Result<String, String> {
        if should_error {
            Err(format!("Error processing: {}", input))
        } else {
            Ok(format!("Success: {}", input))
        }
    }

    let mut group = c.benchmark_group("error_handling");

    group.bench_function("success_path", |b| {
        b.iter(|| error_prone_tool(false, "test input".to_string()));
    });

    group.bench_function("error_path", |b| {
        b.iter(|| error_prone_tool(true, "test input".to_string()));
    });

    group.finish();
}

/// Benchmark tool with optional parameters
fn bench_optional_parameters(c: &mut Criterion) {
    #[tool]
    fn optional_params_tool(
        required: String,
        optional1: Option<String>,
        optional2: Option<i32>,
        optional3: Option<bool>,
    ) -> String {
        format!(
            "required: {}, opt1: {:?}, opt2: {:?}, opt3: {:?}",
            required, optional1, optional2, optional3
        )
    }

    let mut group = c.benchmark_group("optional_parameters");

    group.bench_function("all_none", |b| {
        b.iter(|| optional_params_tool("test".to_string(), None, None, None));
    });

    group.bench_function("all_some", |b| {
        b.iter(|| {
            optional_params_tool(
                "test".to_string(),
                Some("optional".to_string()),
                Some(42),
                Some(true),
            )
        });
    });

    group.bench_function("mixed", |b| {
        b.iter(|| {
            optional_params_tool(
                "test".to_string(),
                Some("optional".to_string()),
                None,
                Some(false),
            )
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    #[tool]
    fn string_heavy_tool(inputs: Vec<String>) -> String {
        inputs.join(", ")
    }

    #[tool]
    fn clone_heavy_tool(input: String, count: usize) -> Vec<String> {
        (0..count).map(|i| format!("{}-{}", input, i)).collect()
    }

    let test_strings: Vec<String> = (0..100).map(|i| format!("test_string_{}", i)).collect();

    let mut group = c.benchmark_group("memory_allocation");

    group.bench_function("string_joining", |b| {
        b.iter(|| string_heavy_tool(test_strings.clone()));
    });

    for count in &[10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("string_cloning", count),
            count,
            |b, &count| {
                b.iter(|| clone_heavy_tool("base".to_string(), count));
            },
        );
    }

    group.finish();
}

/// Benchmark function call overhead
fn bench_function_call_overhead(c: &mut Criterion) {
    // Direct function (baseline)
    fn direct_add(a: i32, b: i32) -> i32 {
        a + b
    }

    // Tool function (with macro overhead)
    #[tool]
    fn tool_add(a: i32, b: i32) -> i32 {
        a + b
    }

    let mut group = c.benchmark_group("call_overhead");

    group.bench_function("direct_function", |b| {
        b.iter(|| direct_add(42, 58));
    });

    group.bench_function("tool_function", |b| {
        b.iter(|| tool_add(42, 58));
    });

    group.finish();
}

/// Benchmark compilation-time factors that affect runtime
fn bench_generated_code_efficiency(c: &mut Criterion) {
    // Test different tool complexities to see if macro-generated code scales well

    #[tool]
    fn minimal_tool() -> i32 {
        42
    }

    #[tool]
    fn medium_tool(a: i32, b: String, c: bool) -> String {
        format!("{}-{}-{}", a, b, c)
    }

    #[tool]
    fn complex_tool(
        nums: Vec<i32>,
        map: std::collections::HashMap<String, String>,
        opt: Option<String>,
    ) -> usize {
        nums.len() + map.len() + opt.map(|s| s.len()).unwrap_or(0)
    }

    let test_nums = vec![1, 2, 3, 4, 5];
    let mut test_map = std::collections::HashMap::new();
    test_map.insert("key1".to_string(), "value1".to_string());
    test_map.insert("key2".to_string(), "value2".to_string());

    let mut group = c.benchmark_group("generated_code_efficiency");

    group.bench_function("minimal_tool", |b| {
        b.iter(minimal_tool);
    });

    group.bench_function("medium_tool", |b| {
        b.iter(|| medium_tool(42, "test".to_string(), true));
    });

    group.bench_function("complex_tool", |b| {
        b.iter(|| {
            complex_tool(
                test_nums.clone(),
                test_map.clone(),
                Some("optional".to_string()),
            )
        });
    });

    group.finish();
}

/// Benchmark concurrent tool execution
fn bench_concurrent_execution(c: &mut Criterion) {
    use std::thread;

    #[tool]
    fn concurrent_safe_tool(input: i32) -> i32 {
        // CPU-intensive work that's safe for concurrent execution
        let mut result = input;
        for _ in 0..1000 {
            result = result.wrapping_mul(17).wrapping_add(1);
        }
        result
    }

    let mut group = c.benchmark_group("concurrent_execution");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("sequential", |b| {
        b.iter(|| {
            for i in 0..10 {
                concurrent_safe_tool(i);
            }
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| thread::spawn(move || concurrent_safe_tool(i)))
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_tool_performance,
    bench_complex_parameter_tools,
    bench_async_tool_performance,
    bench_many_parameters_tool,
    bench_json_performance,
    bench_error_handling_performance,
    bench_optional_parameters,
    bench_memory_allocation,
    bench_function_call_overhead,
    bench_generated_code_efficiency,
    bench_concurrent_execution
);

criterion_main!(benches);
