//! Canister performance profiling command

use super::*;
use anyhow::{anyhow, Result};
use colored::Colorize;
use ic_agent::Agent;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{timeout, Duration};

/// Execute canister profiling command
pub async fn execute(
    canister_id: String,
    duration_secs: u64,
    network: String,
    concurrency: usize,
) -> Result<()> {
    println!(
        "{}",
        "🔍 Profiling Canister Performance".bright_cyan().bold()
    );
    println!("Canister ID: {}", canister_id.bright_yellow());
    println!("Duration: {}s", duration_secs.to_string().bright_blue());
    println!("Network: {}", network.bright_green());
    println!("Concurrency: {}", concurrency.to_string().bright_magenta());
    println!();

    // Create IC agent
    let agent = create_agent(&network).await?;

    // Validate canister exists and is accessible
    validate_canister(&agent, &canister_id).await?;

    println!("⚡ Starting performance profiling...\n");

    // Run profiling
    let metrics = profile_canister(&agent, &canister_id, duration_secs, concurrency).await?;

    // Display results
    display_profiling_results(&metrics, &canister_id);

    Ok(())
}

/// Create IC agent for the specified network
async fn create_agent(network: &str) -> Result<Agent> {
    let url = match network {
        "local" => "http://localhost:4943",
        "ic" => "https://ic0.app",
        custom => custom, // Allow custom URLs
    };

    println!("🌐 Connecting to: {}", url.bright_blue());

    // For local development, we'll use anonymous identity
    // For IC mainnet, we would need proper identity management
    let agent = Agent::builder()
        .with_url(url)
        .build()
        .map_err(|e| anyhow!("Failed to create IC agent: {}", e))?;

    // Fetch root key for local replica
    if network == "local" {
        agent
            .fetch_root_key()
            .await
            .map_err(|e| anyhow!("Failed to fetch root key: {}", e))?;
    }

    Ok(agent)
}

/// Validate that the canister exists and is accessible
async fn validate_canister(agent: &Agent, canister_id: &str) -> Result<()> {
    let principal = canister_id
        .parse()
        .map_err(|e| anyhow!("Invalid canister ID '{}': {}", canister_id, e))?;

    println!("🔍 Validating canister access...");

    // Try to get canister status
    match agent
        .read_state_canister_info(principal, "module_hash")
        .await
    {
        Ok(_) => {
            println!("✅ Canister is accessible");
            Ok(())
        }
        Err(e) => {
            // Try a simple query call instead
            match timeout(
                Duration::from_secs(5),
                agent.query(&principal, "list_tools").call(),
            )
            .await
            {
                Ok(Ok(_)) => {
                    println!("✅ Canister is responsive");
                    Ok(())
                }
                _ => {
                    println!("⚠️  Canister validation inconclusive, continuing anyway...");
                    println!("   Original error: {}", e);
                    Ok(())
                }
            }
        }
    }
}

/// Profile canister performance
async fn profile_canister(
    agent: &Agent,
    canister_id: &str,
    duration_secs: u64,
    concurrency: usize,
) -> Result<PerformanceMetrics> {
    let principal = canister_id.parse().unwrap();
    let end_time = Instant::now() + Duration::from_secs(duration_secs);

    let mut response_times = Vec::new();
    let mut errors = Vec::new();
    let total_bytes = Arc::new(AtomicUsize::new(0));

    println!(
        "🚀 Starting {} concurrent workers for {}s...",
        concurrency, duration_secs
    );

    // Simple approach: spawn concurrent tasks
    let mut tasks = Vec::new();

    for worker_id in 0..concurrency {
        let agent_clone = agent.clone();
        let total_bytes_clone = total_bytes.clone();

        let task = tokio::spawn(async move {
            let mut worker_response_times = Vec::new();
            let mut worker_errors = Vec::new();
            let mut request_count = 0;

            while Instant::now() < end_time {
                let start = Instant::now();
                match make_test_request(&agent_clone, principal).await {
                    Ok(response) => {
                        let elapsed = start.elapsed();
                        worker_response_times.push(elapsed);
                        total_bytes_clone.fetch_add(response.len(), Ordering::Relaxed);
                    }
                    Err(e) => {
                        worker_errors.push(e);
                    }
                }

                request_count += 1;

                // Small delay to prevent overwhelming the canister
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Progress update every 50 requests
                if request_count % 50 == 0 {
                    let elapsed_secs = start.elapsed().as_secs();
                    if elapsed_secs > 0 {
                        println!(
                            "Worker {}: {} requests, {:.1} req/s",
                            worker_id,
                            request_count,
                            request_count as f64 / elapsed_secs as f64
                        );
                    }
                }
            }

            (worker_response_times, worker_errors)
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    println!("⏳ Waiting for workers to complete...");

    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok((mut worker_times, mut worker_errors)) => {
                response_times.append(&mut worker_times);
                errors.append(&mut worker_errors);
                println!("✅ Worker {} completed", i);
            }
            Err(e) => {
                println!("❌ Worker {} failed: {}", i, e);
            }
        }
    }

    println!("🏁 Profiling completed");

    let total_bytes_transferred = total_bytes.load(Ordering::Relaxed);
    let metrics = utils::calculate_metrics(
        &response_times,
        errors.len(),
        total_bytes_transferred as u64,
    );

    Ok(metrics)
}

/// Make a test request to the canister
async fn make_test_request(agent: &Agent, principal: candid::Principal) -> Result<Vec<u8>, String> {
    // Try to call list_tools method (common MCP method)
    let query_call = agent.query(&principal, "list_tools").call();

    let result = timeout(Duration::from_secs(10), query_call).await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(format!("Canister error: {}", e)),
        Err(_) => Err("Request timeout".to_string()),
    }
}

/// Display profiling results
fn display_profiling_results(metrics: &PerformanceMetrics, canister_id: &str) {
    println!("\n{}", "📊 Profiling Results".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_blue());

    println!("🎯 {}: {}", "Canister".bold(), canister_id.bright_yellow());
    println!();

    // Performance metrics
    println!("⚡ {}", "Performance Metrics".bright_green().bold());
    println!(
        "   Requests/sec:     {:.2}",
        metrics.requests_per_second.to_string().bright_cyan()
    );
    println!(
        "   Average latency:  {}",
        utils::format_duration(metrics.average_response_time).bright_yellow()
    );
    println!(
        "   Min latency:      {}",
        utils::format_duration(metrics.min_response_time).bright_green()
    );
    println!(
        "   Max latency:      {}",
        utils::format_duration(metrics.max_response_time).bright_red()
    );
    println!(
        "   95th percentile:  {}",
        utils::format_duration(metrics.p95_response_time).bright_magenta()
    );
    println!();

    // Error rate
    if metrics.error_rate > 0.0 {
        let error_color = if metrics.error_rate > 0.05 {
            "bright_red"
        } else {
            "yellow"
        };
        println!(
            "⚠️  Error rate:       {:.2}%",
            (metrics.error_rate * 100.0).to_string().color(error_color)
        );
    } else {
        println!("✅ Error rate:       {}%", "0.00".bright_green());
    }

    // Throughput
    if metrics.throughput_mb_per_sec > 0.0 {
        println!(
            "📈 Throughput:       {} MB/s",
            format!("{:.2}", metrics.throughput_mb_per_sec).bright_blue()
        );
    }

    println!();

    // Performance assessment
    assess_performance(metrics);
}

/// Provide performance assessment and recommendations
fn assess_performance(metrics: &PerformanceMetrics) {
    println!("🎯 {}", "Performance Assessment".bright_cyan().bold());

    // Response time assessment
    let avg_ms = metrics.average_response_time.as_millis();
    match avg_ms {
        0..=100 => {
            println!(
                "✅ {}Response time: Excellent (<100ms)",
                "✨ ".bright_green()
            );
        }
        101..=500 => {
            println!("🟢 Response time: Good (100-500ms)");
        }
        501..=1000 => {
            println!("🟡 Response time: Fair (500ms-1s)");
        }
        _ => {
            println!("🔴 Response time: Needs improvement (>1s)");
        }
    }

    // Throughput assessment
    if metrics.requests_per_second > 1000.0 {
        println!(
            "✅ {}Throughput: Excellent (>1000 req/s)",
            "✨ ".bright_green()
        );
    } else if metrics.requests_per_second > 500.0 {
        println!("🟢 Throughput: Good (500-1000 req/s)");
    } else if metrics.requests_per_second > 100.0 {
        println!("🟡 Throughput: Fair (100-500 req/s)");
    } else {
        println!("🔴 Throughput: Needs improvement (<100 req/s)");
    }

    // Error rate assessment
    if metrics.error_rate == 0.0 {
        println!(
            "✅ {}Reliability: Perfect (0% errors)",
            "✨ ".bright_green()
        );
    } else if metrics.error_rate < 0.01 {
        println!("🟢 Reliability: Excellent (<1% errors)");
    } else if metrics.error_rate < 0.05 {
        println!("🟡 Reliability: Acceptable (1-5% errors)");
    } else {
        println!("🔴 Reliability: Needs attention (>5% errors)");
    }

    println!();

    // Recommendations
    println!("💡 {}", "Recommendations".bright_yellow().bold());

    if avg_ms > 500 {
        println!("   • Consider optimizing query methods for faster response times");
        println!("   • Review canister memory usage and garbage collection");
    }

    if metrics.requests_per_second < 100.0 {
        println!("   • Profile individual method performance");
        println!("   • Consider using stable memory for better performance");
    }

    if metrics.error_rate > 0.01 {
        println!("   • Investigate error patterns and add better error handling");
        println!("   • Check canister cycles and resource limits");
    }

    if metrics.p95_response_time.as_millis() > metrics.average_response_time.as_millis() * 3 {
        println!("   • High response time variance detected - investigate outliers");
        println!("   • Consider implementing request queuing or rate limiting");
    }
}
