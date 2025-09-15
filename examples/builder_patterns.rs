//! Example demonstrating builder pattern alternatives to macros
//!
//! This example shows how to create MCP tools, servers, and storage configurations
//! using the programmatic builder pattern instead of macros.

use icarus_core::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Builder Pattern Examples for Icarus SDK");
    println!();

    // Example 1: Creating tools with ToolBuilder
    println!("1️⃣  Creating tools with ToolBuilder:");
    tool_builder_example().await?;

    println!();

    // Example 2: Creating servers with ServerBuilder
    println!("2️⃣  Creating servers with ServerBuilder:");
    server_builder_example().await?;

    println!();

    // Example 3: Creating storage with StorageBuilder
    println!("3️⃣  Creating storage with StorageBuilder:");
    storage_builder_example()?;

    println!();
    println!("✅ All builder pattern examples completed successfully!");

    Ok(())
}

/// Demonstrate creating tools with the builder pattern
async fn tool_builder_example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a simple echo tool
    let echo_tool = ToolBuilder::new("echo")
        .description("Echo back the input with optional prefix")
        .parameter("message", "string", "Message to echo", true)
        .parameter("prefix", "string", "Optional prefix to add", false)
        .handler(|args| async move {
            let message = args["message"].as_str().unwrap_or("");
            let prefix = args.get("prefix").and_then(|p| p.as_str()).unwrap_or("");

            let result = if prefix.is_empty() {
                message.to_string()
            } else {
                format!("{}: {}", prefix, message)
            };

            Ok(json!({ "result": result }))
        })
        .build()?;

    println!("   ✅ Created echo tool: {}", echo_tool.info().name);

    // Test the echo tool
    let response = echo_tool
        .execute(json!({
            "message": "Hello, World!",
            "prefix": "Echo"
        }))
        .await?;

    println!("   📝 Test result: {}", response["result"]);

    // Create a math tool with enum parameters
    let math_tool = ToolBuilder::new("math")
        .description("Perform basic math operations")
        .parameter("a", "number", "First number", true)
        .parameter("b", "number", "Second number", true)
        .enum_parameter(
            "operation",
            "Math operation to perform",
            vec![
                "add".to_string(),
                "subtract".to_string(),
                "multiply".to_string(),
                "divide".to_string(),
            ],
            true,
        )
        .handler(|args| async move {
            let a = args["a"].as_f64().ok_or(IcarusError::Other("Invalid number for 'a'".to_string()))?;
            let b = args["b"].as_f64().ok_or(IcarusError::Other("Invalid number for 'b'".to_string()))?;
            let op = args["operation"].as_str().ok_or(IcarusError::Other("Invalid operation".to_string()))?;

            let result = match op {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(IcarusError::Other("Division by zero".to_string()));
                    }
                    a / b
                }
                _ => return Err(IcarusError::Other("Unknown operation".to_string())),
            };

            Ok(json!({ "result": result }))
        })
        .build()?;

    println!("   ✅ Created math tool: {}", math_tool.info().name);

    // Test the math tool
    let response = math_tool
        .execute(json!({
            "a": 10.0,
            "b": 5.0,
            "operation": "multiply"
        }))
        .await?;

    println!("   🧮 Math result: {}", response["result"]);

    // Create a tool with default parameters
    let greeting_tool = ToolBuilder::new("greet")
        .description("Generate a greeting message")
        .parameter("name", "string", "Name to greet", true)
        .parameter_with_default("greeting", "string", "Greeting word", json!("Hello"))
        .parameter_with_default("punctuation", "string", "Punctuation to use", json!("!"))
        .handler(|args| async move {
            let name = args["name"].as_str().unwrap_or("World");
            let greeting = args
                .get("greeting")
                .and_then(|g| g.as_str())
                .unwrap_or("Hello");
            let punct = args
                .get("punctuation")
                .and_then(|p| p.as_str())
                .unwrap_or("!");

            Ok(json!({
                "result": format!("{} {}{}", greeting, name, punct),
                "used_defaults": json!({
                    "greeting": args.get("greeting").is_none(),
                    "punctuation": args.get("punctuation").is_none()
                })
            }))
        })
        .build()?;

    println!("   ✅ Created greeting tool: {}", greeting_tool.info().name);

    // Test with defaults
    let response = greeting_tool.execute(json!({ "name": "Alice" })).await?;

    println!("   👋 Greeting result: {}", response["result"]);
    println!("   🎯 Used defaults: {}", response["used_defaults"]);

    Ok(())
}

/// Demonstrate creating servers with the builder pattern
async fn server_builder_example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create some tools first
    let tool1 = ToolBuilder::new("status")
        .description("Get server status")
        .handler(|_| async move {
            Ok(json!({
                "status": "online",
                "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                "version": "1.0.0"
            }))
        })
        .build()?;

    let tool2 = ToolBuilder::new("info")
        .description("Get server information")
        .handler(|_| async move {
            Ok(json!({
                "name": "Example MCP Server",
                "description": "Built with builder patterns",
                "features": ["status", "info", "custom_metadata"]
            }))
        })
        .build()?;

    // Build a server with these tools
    let server = ServerBuilder::new("Example MCP Server")
        .version("2.1.0")
        .description("A demonstration server built with builder patterns")
        .add_tool(tool1)
        .add_tool(tool2)
        .metadata("author", json!("Icarus SDK"))
        .metadata("license", json!("BSL-1.1"))
        .metadata("repository", json!("https://github.com/example/server"))
        .metadata("capabilities", json!(["tools", "resources", "logging"]))
        .build();

    println!("   ✅ Created server: {}", server.name);
    println!("   📦 Version: {}", server.version);
    println!("   🔧 Tools: {}", server.tools().len());

    // Display server info
    let info = server.info();
    println!("   📋 Server info:");
    println!("      Name: {}", info["name"]);
    println!("      Description: {}", info["description"]);
    println!("      Custom metadata:");
    if let Some(author) = info.get("author") {
        println!("        Author: {}", author);
    }
    if let Some(license) = info.get("license") {
        println!("        License: {}", license);
    }

    // Test finding tools
    if let Some(status_tool) = server.find_tool("status") {
        println!("   🔍 Found tool: {}", status_tool.info().name);

        let response = status_tool.execute(json!({})).await?;
        println!("   ⚡ Tool response: {}", response);
    }

    Ok(())
}

/// Demonstrate creating storage configurations with the builder pattern
fn storage_builder_example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a storage configuration
    let storage = StorageBuilder::new()
        .add_map::<String, serde_json::Value>("users") // Memory ID 0
        .add_map::<u64, String>("sessions") // Memory ID 1
        .add_cell::<u64>("counter", json!(0)) // Memory ID 2
        .add_cell::<String>("server_name", json!("My Server")) // Memory ID 3
        .add_map_with_id::<String, Vec<u8>>("files", 10) // Explicit memory ID
        .build();

    println!("   ✅ Created storage configuration");
    println!(
        "   📊 Maps: {}, Cells: {}",
        storage.maps.len(),
        storage.cells.len()
    );

    // Show configuration summary
    let summary = storage.summary();
    println!("   📋 Storage summary: {}", summary);

    // Generate the equivalent code
    println!("\n   📝 Generated storage code:");
    println!("{}", storage.generate_code());

    // Show individual configurations
    println!("   📚 Storage components:");
    for map in &storage.maps {
        println!(
            "      Map '{}': {} -> {} (Memory ID: {})",
            map.name, map.key_type, map.value_type, map.memory_id
        );
    }
    for cell in &storage.cells {
        println!(
            "      Cell '{}': {} = {} (Memory ID: {})",
            cell.name, cell.value_type, cell.default_value, cell.memory_id
        );
    }

    Ok(())
}

/// Example of combining all builders for a complete MCP server setup
#[allow(dead_code)]
async fn complete_server_example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Complete Server Setup Example:");

    // 1. Define storage
    let storage = StorageBuilder::new()
        .add_map::<String, serde_json::Value>("data")
        .add_cell::<u64>("request_count", json!(0))
        .build();

    println!(
        "   📦 Storage configured with {} maps and {} cells",
        storage.maps.len(),
        storage.cells.len()
    );

    // 2. Create tools
    let get_tool = ToolBuilder::new("get_data")
        .description("Retrieve data by key")
        .parameter("key", "string", "Data key to retrieve", true)
        .handler(|args| async move {
            let key = args["key"].as_str().unwrap_or("");
            // In real implementation, this would query stable storage
            Ok(json!({
                "key": key,
                "value": format!("Data for key: {}", key),
                "found": true
            }))
        })
        .build()?;

    let set_tool = ToolBuilder::new("set_data")
        .description("Store data by key")
        .parameter("key", "string", "Data key", true)
        .parameter("value", "string", "Data value", true)
        .handler(|args| async move {
            let key = args["key"].as_str().unwrap_or("");
            let value = args["value"].as_str().unwrap_or("");
            // In real implementation, this would update stable storage
            Ok(json!({
                "success": true,
                "key": key,
                "stored": value
            }))
        })
        .build()?;

    let stats_tool = ToolBuilder::new("stats")
        .description("Get server statistics")
        .handler(|_| async move {
            // In real implementation, this would read from stable storage
            Ok(json!({
                "total_requests": 42,
                "data_items": 15,
                "uptime_seconds": 3600
            }))
        })
        .build()?;

    // 3. Build the complete server
    let server = ServerBuilder::new("Data Storage MCP Server")
        .version("1.2.0")
        .description("A complete MCP server with persistent storage")
        .add_tool(get_tool)
        .add_tool(set_tool)
        .add_tool(stats_tool)
        .metadata("storage_type", json!("stable_memory"))
        .metadata("max_data_size", json!("1MB"))
        .build();

    println!("   🚀 Complete server built:");
    println!("      Name: {}", server.name);
    println!("      Tools: {}", server.tools().len());
    println!(
        "      Storage: {} components",
        storage.maps.len() + storage.cells.len()
    );

    // Test the complete setup
    if let Some(stats_tool) = server.find_tool("stats") {
        let response = stats_tool.execute(json!({})).await?;
        println!("   📊 Server stats: {}", response);
    }

    Ok(())
}
