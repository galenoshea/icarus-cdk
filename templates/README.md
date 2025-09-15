# Icarus SDK Templates

Production-ready templates to kickstart your MCP server development.

## Available Templates

### 📊 Data Manager
**Path**: `data-manager/`
**Use Case**: Structured data management with full CRUD operations

**Features**:
- Full CRUD operations with validation
- Advanced search and filtering
- User management and permissions
- Analytics and reporting
- Data export and backup

**Best For**: Content management systems, document stores, inventory management, user data platforms

### ⏰ Task Scheduler
**Path**: `task-scheduler/`
**Use Case**: Time-based operations and background processing

**Features**:
- Flexible scheduling (cron, interval, daily, weekly, monthly)
- Retry logic and error handling
- Task monitoring and history
- Pre-built task templates
- Resource management

**Best For**: Automated backups, data cleanup, notifications, periodic reports, system maintenance

### 🌐 API Gateway
**Path**: `api-gateway/`
**Use Case**: External API integrations and service connectors

**Features**:
- HTTP client with multiple auth methods
- Response caching and rate limiting
- Request transformation
- Error handling and retries
- Performance monitoring

**Best For**: Third-party service integrations, webhook handlers, API aggregation, data synchronization

## Using Templates

### Create from Template
```bash
# Create new project from template
icarus new my-project --template data-manager
icarus new my-scheduler --template task-scheduler
icarus new my-gateway --template api-gateway
```

### Deploy Template
```bash
cd my-project
dfx start --background
icarus build
icarus deploy --network local
```

### Connect to Claude Desktop
```json
{
  "mcpServers": {
    "my-project": {
      "command": "icarus",
      "args": ["bridge", "start", "--canister-id", "YOUR_CANISTER_ID"]
    }
  }
}
```

## Template Architecture

All templates follow Icarus SDK best practices:

- **Stable Storage**: Persistent data across canister upgrades
- **Type Safety**: Full TypeScript-like type safety with Candid
- **Error Handling**: Comprehensive error handling and validation
- **Security**: Principal-based authentication and authorization
- **Performance**: Optimized for ICP's pay-per-use model
- **Monitoring**: Built-in logging and metrics collection

## Customization Guide

### Extending Templates
1. **Add Custom Data Models**: Define new structs with `#[derive(IcarusStorable)]`
2. **Create New Tools**: Add functions with `#[icarus_tool("description")]`
3. **Implement Business Logic**: Add domain-specific functionality
4. **Configure Storage**: Adjust stable memory layout as needed

### Common Patterns
```rust
// Custom data model
#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone)]
pub struct MyData {
    pub id: String,
    pub content: String,
    pub created_at: u64,
}

// Custom MCP tool
#[icarus_tool("Process custom data")]
pub async fn process_data(data: MyData) -> Result<String, String> {
    // Your business logic here
    Ok("Processed successfully".to_string())
}

// Storage configuration
stable_storage! {
    memory 0: {
        my_data: Map<String, MyData> = Map::init();
    }
}
```

## Contributing Templates

To contribute a new template:

1. **Create Template Directory**: `templates/my-template/`
2. **Add Core Files**: `Cargo.toml`, `src/lib.rs`, `README.md`
3. **Follow Conventions**: Use established patterns and best practices
4. **Document Thoroughly**: Provide comprehensive README with examples
5. **Test Deployment**: Verify template works end-to-end
6. **Submit PR**: Include template in this index

### Template Requirements
- Production-ready code quality
- Comprehensive error handling
- Full documentation with examples
- Follows Icarus SDK best practices
- Includes deployment instructions
- Provides Claude Desktop integration guide

## License

All templates are provided under the BSL-1.1 license. See the main SDK license for details.