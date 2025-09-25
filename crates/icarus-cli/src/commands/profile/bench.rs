//! Benchmark execution command

use super::*;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute benchmark command
pub async fn execute(filter: Option<String>, output: Option<String>, html: bool) -> Result<()> {
    println!(
        "{}",
        "🚀 Running Icarus CDK Benchmarks".bright_cyan().bold()
    );

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        return Err(anyhow!("No Cargo.toml found. Please run this command from an Icarus project root or the CDK directory."));
    }

    // Build benchmark command
    let mut cmd = Command::new("cargo");
    cmd.arg("bench");

    // Add filter if specified
    if let Some(filter) = filter {
        println!("📝 Filtering benchmarks: {}", filter.bright_yellow());
        cmd.arg(&filter);
    }

    // Set benchmark output format
    if html {
        cmd.env("CRITERION_FORMAT", "html");
        println!("📊 Generating HTML report");
    }

    println!("⚡ Running benchmarks...\n");

    // Execute benchmark
    let output_result = cmd
        .current_dir(&current_dir)
        .output()
        .map_err(|e| anyhow!("Failed to run benchmarks: {}", e))?;

    if !output_result.status.success() {
        eprintln!("{}", "❌ Benchmark execution failed:".bright_red());
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
        return Err(anyhow!(
            "Benchmarks failed with exit code: {}",
            output_result.status.code().unwrap_or(-1)
        ));
    }

    // Display benchmark output
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    println!("{}", stdout);

    // Save results if output file specified
    if let Some(output_path) = output {
        save_benchmark_results(&output_path, &stdout).await?;
        println!("\n💾 Results saved to: {}", output_path.bright_green());
    }

    // Check for HTML report
    let target_dir = find_target_directory(&current_dir)?;
    let html_report = target_dir
        .join("criterion")
        .join("reports")
        .join("index.html");

    if html && html_report.exists() {
        println!(
            "\n📊 HTML report available at: {}",
            html_report.display().to_string().bright_blue().underline()
        );

        // Try to open in browser on macOS/Linux/Windows
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&html_report).spawn();

        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&html_report).spawn();

        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd")
            .args(["/c", "start", &html_report.to_string_lossy()])
            .spawn();
    }

    // Show benchmark summary
    show_benchmark_summary(&stdout);

    println!(
        "\n✅ {}",
        "Benchmarks completed successfully!".bright_green().bold()
    );

    Ok(())
}

/// Find the target directory for build artifacts
fn find_target_directory(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let target = current.join("target");
        if target.exists() && target.is_dir() {
            return Ok(target);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Default to target in current directory
    Ok(start_dir.join("target"))
}

/// Save benchmark results to file
async fn save_benchmark_results(output_path: &str, results: &str) -> Result<()> {
    let path = PathBuf::from(output_path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Determine format based on file extension
    if output_path.ends_with(".json") {
        // Create JSON format result
        let report = BenchmarkReport {
            results: parse_benchmark_output(results),
            summary: calculate_overall_summary(results),
            environment: EnvironmentInfo::collect(),
        };

        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&path, json)?;
    } else {
        // Save as text
        fs::write(&path, results)?;
    }

    Ok(())
}

/// Parse benchmark output to extract results
fn parse_benchmark_output(output: &str) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    for line in output.lines() {
        if line.contains("time:") && line.contains("ns/iter") {
            if let Some(result) = parse_benchmark_line(line) {
                results.push(result);
            }
        }
    }

    results
}

/// Parse a single benchmark output line
fn parse_benchmark_line(line: &str) -> Option<BenchmarkResult> {
    // Example line: "test_name        time:   [125.67 ns 126.45 ns 127.23 ns]"
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 6 {
        return None;
    }

    let name = parts[0].to_string();

    // Extract the middle time value (typical benchmark result)
    if let Ok(time_ns) = parts[4].parse::<f64>() {
        let duration = Duration::from_nanos(time_ns as u64);

        let metrics = PerformanceMetrics {
            duration,
            requests_per_second: 1_000_000_000.0 / time_ns, // Convert from ns to rps
            average_response_time: duration,
            min_response_time: duration,
            max_response_time: duration,
            p95_response_time: duration,
            error_rate: 0.0,
            throughput_mb_per_sec: 0.0,
        };

        Some(BenchmarkResult {
            name,
            metrics,
            timestamp: chrono::Utc::now(),
        })
    } else {
        None
    }
}

/// Calculate overall summary from benchmark output
fn calculate_overall_summary(output: &str) -> PerformanceMetrics {
    let results = parse_benchmark_output(output);

    if results.is_empty() {
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

    let total_duration: Duration = results.iter().map(|r| r.metrics.duration).sum();
    let average_duration = total_duration / results.len() as u32;

    let min_duration = results
        .iter()
        .map(|r| r.metrics.average_response_time)
        .min()
        .unwrap_or_default();

    let max_duration = results
        .iter()
        .map(|r| r.metrics.average_response_time)
        .max()
        .unwrap_or_default();

    PerformanceMetrics {
        duration: total_duration,
        requests_per_second: results.len() as f64 / total_duration.as_secs_f64(),
        average_response_time: average_duration,
        min_response_time: min_duration,
        max_response_time: max_duration,
        p95_response_time: max_duration, // Simplified
        error_rate: 0.0,
        throughput_mb_per_sec: 0.0,
    }
}

/// Show a summary of benchmark results
fn show_benchmark_summary(output: &str) {
    let results = parse_benchmark_output(output);

    if results.is_empty() {
        println!("\n📊 No benchmark results found");
        return;
    }

    println!("\n📊 {} Benchmark Summary", "=".repeat(50));
    println!(
        "Total benchmarks: {}",
        results.len().to_string().bright_cyan()
    );

    // Find fastest and slowest
    if let Some(fastest) = results
        .iter()
        .min_by_key(|r| r.metrics.average_response_time)
    {
        println!(
            "⚡ Fastest: {} ({})",
            fastest.name.bright_green(),
            utils::format_duration(fastest.metrics.average_response_time).bright_yellow()
        );
    }

    if let Some(slowest) = results
        .iter()
        .max_by_key(|r| r.metrics.average_response_time)
    {
        println!(
            "🐌 Slowest: {} ({})",
            slowest.name.bright_red(),
            utils::format_duration(slowest.metrics.average_response_time).bright_yellow()
        );
    }

    // Calculate average performance
    let total_ops_per_sec: f64 = results.iter().map(|r| r.metrics.requests_per_second).sum();
    let avg_ops_per_sec = total_ops_per_sec / results.len() as f64;

    println!(
        "📈 Average throughput: {:.0} ops/sec",
        avg_ops_per_sec.to_string().bright_blue()
    );
}
