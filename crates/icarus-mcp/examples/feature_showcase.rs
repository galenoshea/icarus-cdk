//! Feature Showcase
//!
//! This example demonstrates the different feature combinations
//! and their optimal usage patterns.

#[cfg(feature = "client")]
use icarus_mcp::McpConfigBuilder;

#[cfg(feature = "server")]
use icarus_mcp::McpServer;

#[cfg(feature = "streaming")]
use icarus_mcp::{CustomSize, DefaultBuffer, Large, Small, StreamingResponse};

use candid::Principal;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Icarus MCP Feature Showcase");
    println!();

    // Feature detection
    println!("📋 Available features:");
    #[cfg(feature = "client")]
    println!("  ✅ client - ICP canister client");
    #[cfg(not(feature = "client"))]
    println!("  ❌ client - disabled");

    #[cfg(feature = "server")]
    println!("  ✅ server - MCP server implementation");
    #[cfg(not(feature = "server"))]
    println!("  ❌ server - disabled");

    #[cfg(feature = "streaming")]
    println!("  ✅ streaming - Large response streaming");
    #[cfg(not(feature = "streaming"))]
    println!("  ❌ streaming - disabled");

    #[cfg(feature = "protocol")]
    println!("  ✅ protocol - MCP protocol handling");
    #[cfg(not(feature = "protocol"))]
    println!("  ❌ protocol - disabled");

    #[cfg(feature = "networking")]
    println!("  ✅ networking - Connection pooling");
    #[cfg(not(feature = "networking"))]
    println!("  ❌ networking - disabled");

    #[cfg(feature = "cli")]
    println!("  ✅ cli - Command-line utilities");
    #[cfg(not(feature = "cli"))]
    println!("  ❌ cli - disabled");

    println!();

    // Client feature demonstration
    #[cfg(feature = "client")]
    {
        println!("🔗 Client Feature Demo");

        // Example with builder pattern
        let config = McpConfigBuilder::new()
            .canister_id(Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")?)
            .ic_url("http://localhost:4943")
            .timeout(Duration::from_secs(30))
            .max_concurrent_requests(10)
            .build()?;

        println!(
            "  📋 Configuration: canister {} on {}",
            config.canister_id, config.ic_url
        );

        // Note: Would create client in real usage
        // let client = CanisterClient::new(config).await?;
        println!("  ✅ Client configuration ready");
        println!();
    }

    // Server feature demonstration
    #[cfg(feature = "server")]
    {
        println!("🖥️ Server Feature Demo");

        // Type-state pattern demonstration
        let _uninitialized_server = McpServer::new();
        println!("  📋 Server created in Uninitialized state");

        // Would connect in real usage:
        // let connected_server = uninitialized_server.connect(config).await?;
        println!("  ✅ Type-safe server transitions available");
        println!();
    }

    // Streaming feature demonstration
    #[cfg(feature = "streaming")]
    {
        println!("📡 Streaming Feature Demo");

        // Different buffer size configurations
        let _small_stream = StreamingResponse::<Small>::new();
        let _default_stream = StreamingResponse::<DefaultBuffer>::new();
        let _large_stream = StreamingResponse::<Large>::new();
        let _custom_stream = StreamingResponse::<CustomSize<1024>>::new();

        println!("  📋 Buffer sizes:");
        println!(
            "    Small:   {} bytes",
            StreamingResponse::<Small>::buffer_size()
        );
        println!(
            "    Default: {} bytes",
            StreamingResponse::<DefaultBuffer>::buffer_size()
        );
        println!(
            "    Large:   {} bytes",
            StreamingResponse::<Large>::buffer_size()
        );
        println!(
            "    Custom:  {} bytes",
            StreamingResponse::<CustomSize<1024>>::buffer_size()
        );

        println!("  ✅ Zero-cost buffer size abstractions");
        println!();
    }

    // Protocol feature demonstration
    #[cfg(feature = "protocol")]
    {
        println!("🔄 Protocol Feature Demo");
        println!("  📋 Trait-based architecture available");
        println!("  ✅ McpProtocol, ToolConverter, CanisterBackend traits");
        println!("  ✅ Dependency injection and testing support");
        println!();
    }

    println!("🎯 Feature showcase complete!");
    println!();
    println!("💡 Tips:");
    println!("  • Use --no-default-features for minimal builds");
    println!("  • Combine features based on your use case");
    println!(
        "  • Check binary size: cargo build --release && du -h target/release/feature_showcase"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_detection() {
        // Compile-time feature detection
        #[cfg(feature = "client")]
        {
            // Client feature is available
        }

        #[cfg(feature = "streaming")]
        {
            // Streaming feature is available
            use icarus_mcp::{DefaultBuffer, Large, Small};
            assert_eq!(StreamingResponse::<Small>::buffer_size(), 4 * 1024);
            assert_eq!(StreamingResponse::<Large>::buffer_size(), 256 * 1024);
            assert_eq!(StreamingResponse::<DefaultBuffer>::buffer_size(), 64 * 1024);
        }
    }
}
