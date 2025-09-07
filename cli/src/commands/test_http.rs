//! Test HTTP outcalls command

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use serde_json::json;
use std::process::Command;

/// Test HTTP outcalls functionality
#[derive(Debug, Parser)]
pub struct TestHttpCmd {
    /// Canister ID to test
    #[arg(long)]
    pub canister_id: String,

    /// Network to use (local or ic)
    #[arg(long, default_value = "local")]
    pub network: String,

    /// URL to fetch
    #[arg(long, default_value = "https://api.github.com/meta")]
    pub url: String,

    /// Test type (get, post, json, weather, crypto)
    #[arg(long, default_value = "get")]
    pub test_type: String,
}

impl TestHttpCmd {
    pub async fn execute(self) -> Result<()> {
        println!("{}", "🌐 Testing HTTP Outcalls".bold().blue());
        println!("  Canister: {}", self.canister_id.yellow());
        println!("  Network:  {}", self.network.yellow());
        println!("  Test:     {}", self.test_type.yellow());
        println!();

        // Build the dfx command based on test type
        let (method, args) = match self.test_type.as_str() {
            "get" => {
                println!("📥 Testing GET request to: {}", self.url.cyan());
                ("fetch_url", format!("'(\"{}\")'", self.url))
            }
            "json" => {
                println!("📊 Testing JSON fetch from: {}", self.url.cyan());
                ("fetch_json", format!("'(\"{}\")'", self.url))
            }
            "post" => {
                let test_data = json!({
                    "test": "data",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "source": "icarus-cli"
                });
                println!("📤 Testing POST to httpbin.org/post");
                println!("   Data: {}", serde_json::to_string_pretty(&test_data)?);
                (
                    "post_data",
                    format!(
                        "'(\"https://httpbin.org/post\", \"{}\")'",
                        test_data.to_string().replace('"', "\\\"")
                    ),
                )
            }
            "weather" => {
                println!("🌤️  Testing weather fetch for London");
                ("fetch_weather", "'(\"London\")'".to_string())
            }
            "crypto" => {
                println!("💰 Testing crypto price fetch for Bitcoin");
                ("fetch_crypto_price", "'(\"bitcoin\")'".to_string())
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown test type: {}. Use: get, json, post, weather, or crypto",
                    self.test_type
                ))
                .context("Invalid test type");
            }
        };

        // Execute the dfx canister call
        println!("\n{} Calling canister method...", "→".green());
        let output = Command::new("dfx")
            .args(&[
                "canister",
                "call",
                &self.canister_id,
                method,
                &args,
                "--network",
                &self.network,
            ])
            .output()
            .context("Failed to execute dfx command")?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            println!(
                "\n{} {}",
                "✅ Success!".green().bold(),
                "Response received:"
            );

            // Try to parse and pretty-print if it's JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(
                result
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .replace("\\\"", "\"")
                    .replace("\\n", "\n")
                    .as_str(),
            ) {
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            } else {
                // Just print the raw result if not JSON
                println!("{}", result);
            }

            println!("\n💡 {}", "Tip:".yellow().bold());
            println!("   You can use the bridge to interact with this canister:");
            println!(
                "   {}",
                format!("icarus bridge start --canister-id {}", self.canister_id).cyan()
            );
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!(
                "\n{} {}",
                "❌ Error:".red().bold(),
                "Failed to call canister"
            );
            println!("{}", error);

            if error.contains("HttpRequestError") {
                println!(
                    "\n{} {}",
                    "ℹ️ Note:".yellow(),
                    "HTTP outcalls require cycles for execution."
                );
                println!("   Make sure your canister has sufficient cycles.");
                println!("   Local canisters get cycles automatically.");
            }
        }

        Ok(())
    }
}

/// Print test instructions
pub fn print_http_test_info() {
    println!("\n{}", "HTTP Outcalls Testing Guide".bold().blue());
    println!("{}", "─".repeat(40));

    println!("\n{}", "Available test types:".bold());
    println!("  • {} - Simple GET request", "get".cyan());
    println!("  • {} - Fetch and parse JSON", "json".cyan());
    println!("  • {} - POST JSON data", "post".cyan());
    println!("  • {} - Fetch weather data", "weather".cyan());
    println!("  • {} - Fetch crypto prices", "crypto".cyan());

    println!("\n{}", "Examples:".bold());
    println!("  # Test GET request");
    println!("  {}", "icarus test http --canister-id <id>".cyan());

    println!("\n  # Test JSON parsing");
    println!(
        "  {}",
        "icarus test http --canister-id <id> --test-type json".cyan()
    );

    println!("\n  # Test POST request");
    println!(
        "  {}",
        "icarus test http --canister-id <id> --test-type post".cyan()
    );

    println!("\n  # Test weather API");
    println!(
        "  {}",
        "icarus test http --canister-id <id> --test-type weather".cyan()
    );

    println!("\n  # Test crypto price API");
    println!(
        "  {}",
        "icarus test http --canister-id <id> --test-type crypto".cyan()
    );

    println!("\n{}", "Note:".yellow().bold());
    println!("  HTTP outcalls consume cycles. Each request costs ~49M cycles");
    println!("  plus additional cycles based on request/response size.");
}
