# Bridge Architecture

Understanding how the Icarus bridge translates between MCP and ICP protocols.

## Overview

The Icarus bridge is a crucial component that enables Claude Desktop (or any MCP client) to communicate with ICP canisters. It acts as a protocol translator, converting MCP's JSON-RPC messages to ICP's Candid calls and vice versa.

## Architecture Diagram

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│ Claude Desktop  │ ──────> │  Icarus Bridge   │ ──────> │  ICP Canister   │
│  (MCP Client)   │ <────── │  (Translator)    │ <────── │ (Pure Backend)  │
└─────────────────┘         └──────────────────┘         └─────────────────┘
     JSON-RPC                  Protocol Layer                  Candid
    over stdio               MCP ↔ ICP Bridge              Standard ICP
```

## Key Design Principles

### 1. Clean Separation
- Canisters have NO knowledge of MCP protocol
- All MCP complexity is handled in the bridge
- Canisters expose standard Candid interfaces

### 2. Stdio Communication
- Bridge communicates with Claude Desktop via stdio (stdin/stdout)
- No network ports or WebSocket servers needed
- Simpler security model

### 3. Dynamic Discovery
- Bridge queries canister's `get_metadata()` function
- Automatically discovers available tools
- No hardcoded tool definitions

## How It Works

### 1. Bridge Initialization

When you run `icarus bridge start --canister-id <id>`:

```rust
// Bridge startup sequence
1. Parse command line arguments
2. Create ICP agent with canister ID
3. Query canister's get_metadata() function
4. Parse tool definitions from metadata
5. Initialize stdio MCP server
6. Start listening for MCP messages
```

### 2. Tool Discovery

The bridge discovers tools dynamically:

```rust
// Query canister for metadata
let metadata_response = canister.query("get_metadata", ()).await?;
let metadata: String = decode_one(&metadata_response)?;
let tools: ToolMetadata = serde_json::from_str(&metadata)?;

// Register tools with MCP server
for tool in tools.tools {
    mcp_server.register_tool(Tool {
        name: tool.name,
        description: tool.description,
        input_schema: tool.input_schema,
    });
}
```

### 3. Message Translation

#### MCP → Candid

When Claude sends a tool call:

```json
// MCP Request
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "memorize",
    "arguments": {
      "content": "Hello world",
      "tags": ["greeting"]
    }
  },
  "id": 1
}
```

Bridge translates to Candid:

```rust
// Translation process
1. Extract tool name and arguments
2. Find corresponding Candid method
3. Convert JSON arguments to Candid types
4. Call canister method
5. Wait for response
```

#### Candid → MCP

When canister responds:

```rust
// Candid Response
Ok("mem_123")

// Translated to MCP
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "mem_123"
      }
    ]
  },
  "id": 1
}
```

## Implementation Details

### Bridge Components

#### 1. MCP Server (stdio)
Uses the `rmcp` crate to implement MCP protocol:

```rust
use rmcp::stdio_server::StdioServer;

let server = StdioServer::new(bridge_handler)
    .with_name("icarus-bridge")
    .with_version(env!("CARGO_PKG_VERSION"));

server.run().await?;
```

#### 2. ICP Client
Communicates with canisters using `ic-agent`:

```rust
use ic_agent::{Agent, AgentBuilder};

let agent = AgentBuilder::default()
    .with_url(ic_host)
    .build()?;

let response = agent.query(&canister_id, method_name)
    .with_arg(candid_args)
    .call()
    .await?;
```

#### 3. Protocol Translator
Core translation logic:

```rust
pub struct ProtocolTranslator {
    canister_client: CanisterClient,
    tool_registry: HashMap<String, ToolDefinition>,
}

impl ProtocolTranslator {
    pub async fn translate_mcp_call(&self, 
        tool_name: &str, 
        args: serde_json::Value
    ) -> Result<serde_json::Value> {
        // Get tool definition
        let tool = self.tool_registry.get(tool_name)
            .ok_or("Unknown tool")?;
        
        // Convert JSON to Candid
        let candid_args = json_to_candid(&args, &tool.param_types)?;
        
        // Call canister
        let response = self.canister_client
            .call(&tool.candid_method, candid_args)
            .await?;
        
        // Convert response back to JSON
        candid_to_json(response)
    }
}
```

### Type Mapping

The bridge handles type conversions:

| JSON Type | Candid Type | Notes |
|-----------|-------------|-------|
| string | Text | UTF-8 encoded |
| number | Nat/Int/Float | Based on schema |
| boolean | Bool | Direct mapping |
| array | Vec | Homogeneous types |
| object | Record | Named fields |
| null | Opt (None) | Optional values |

### Error Handling

Errors are translated to MCP error responses:

```rust
match canister_call().await {
    Ok(result) => {
        // Success response
        MpcResponse::success(result)
    }
    Err(e) => {
        // Error response with details
        MpcResponse::error(ErrorCode::ToolError, e.to_string())
    }
}
```

## Security Considerations

### 1. Local Execution
- Bridge runs locally on user's machine
- No centralized servers or middlemen
- User controls their own bridge

### 2. Canister Authentication
- Bridge can authenticate with Internet Identity
- Supports principal-based access control
- Canisters can verify caller identity

### 3. Data Privacy
- All communication is local (stdio)
- No data leaves user's machine via bridge
- Direct connection to ICP network

## Advanced Features

### 1. Batch Operations

The bridge can batch multiple tool calls:

```rust
// Process multiple calls efficiently
let futures: Vec<_> = calls.iter()
    .map(|call| translate_and_execute(call))
    .collect();

let results = futures::future::join_all(futures).await;
```

### 2. Streaming Responses

For large data, the bridge supports streaming:

```rust
// Stream large results back to Claude
for chunk in large_result.chunks(1024) {
    send_partial_response(chunk).await?;
}
```

### 3. Caching

Frequently accessed data can be cached:

```rust
// Cache query results
if tool.is_query {
    if let Some(cached) = cache.get(&cache_key) {
        return Ok(cached.clone());
    }
}

let result = execute_call().await?;
cache.insert(cache_key, result.clone());
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug icarus bridge start --canister-id <id>
```

### Common Issues

#### 1. "Failed to connect to canister"
- Check canister ID is correct
- Verify network connectivity
- Ensure canister is deployed

#### 2. "Tool not found"
- Verify canister has `get_metadata()` function
- Check tool names match exactly
- Ensure metadata format is correct

#### 3. "Type conversion error"
- Check argument types match Candid interface
- Verify JSON schema in metadata
- Look for null/undefined values

### Monitoring

The bridge logs important events:

```
[INFO] Bridge started for canister: rrkah-fqaaa-aaaaa-aaaaq-cai
[INFO] Discovered 5 tools from metadata
[DEBUG] Received MCP request: tools/list
[DEBUG] Calling canister method: list
[INFO] Tool call completed in 150ms
```

## Performance

### Optimization Strategies

1. **Query vs Update**: Bridge knows which calls are read-only
2. **Parallel Execution**: Independent calls run concurrently
3. **Connection Pooling**: Reuses HTTP connections to IC
4. **Response Caching**: Optional caching for query methods

### Benchmarks

Typical latencies:
- Local network: 10-50ms
- IC mainnet: 100-500ms
- Tool discovery: One-time 200ms overhead

## Future Enhancements

### Planned Features

1. **WebSocket Support**: Alternative to stdio
2. **Multi-Canister**: Bridge connecting to multiple canisters
3. **State Synchronization**: Optimistic updates
4. **Plugin System**: Custom protocol extensions

### Under Consideration

1. **P2P Mode**: Direct canister-to-canister communication
2. **Compressed Transport**: For large data transfers
3. **GraphQL Interface**: Alternative query language
4. **Real-time Subscriptions**: For live data updates