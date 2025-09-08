# Icarus CLI Command Reference

Complete reference for all Icarus CLI commands.

## Global Options

These options can be used with any command:

- `--help, -h` - Show help for a command
- `--version, -V` - Show version information
- `--verbose, -v` - Enable verbose output
- `--quiet, -q` - Suppress non-error output

## Commands

### `icarus new`

Create a new Icarus MCP server project.

**Usage:**
```bash
icarus new <name> [options]
```

**Arguments:**
- `<name>` - Project name (required)

**Options:**
- `--path <path>` - Directory to create project in (default: current directory)
- `--local-sdk <path>` - Use local SDK instead of crates.io version
- `--with-tests` - Include test files and dependencies
- `--template <name>` - Use a specific template (default: memory)

**Examples:**
```bash
# Create a basic project
icarus new my-server

# Create with tests
icarus new my-server --with-tests

# Use local SDK for development
icarus new my-server --local-sdk ../icarus-sdk

# Create in specific directory
icarus new my-server --path ~/projects
```

### `icarus deploy`

Deploy the project to ICP network.

**Usage:**
```bash
icarus deploy [options]
```

**Options:**
- `--network <network>` - Target network: local, playground, ic (default: local)
- `--force` - Force deployment even if no changes
- `--upgrade` - Upgrade existing canister instead of fresh deploy
- `--cycles <amount>` - Cycles to initialize canister with
- `--with-cycles <amount>` - Cycles for canister operations
- `--compute-allocation <0-100>` - Guaranteed compute allocation percentage
- `--memory-allocation <bytes>` - Memory allocation (e.g., 2GB)

**Examples:**
```bash
# Deploy to local network
icarus deploy --network local

# Deploy to mainnet with cycles
icarus deploy --network ic --cycles 1000000000000

# Upgrade existing deployment
icarus deploy --upgrade

# Deploy with specific resources
icarus deploy --compute-allocation 10 --memory-allocation 4GB
```

### `icarus test`

Run tests with progressive testing strategy.

**Usage:**
```bash
icarus test [options]
```

**Options:**
- `--level <level>` - Test level: 1 (unit), 2 (canister), 3 (mcp), 4 (integration), all
- `--filter <pattern>` - Run only tests matching pattern
- `--verbose` - Show detailed test output
- `--nocapture` - Don't capture test output

**Examples:**
```bash
# Run all tests
icarus test

# Run only unit tests
icarus test --level 1

# Run specific test
icarus test --filter test_memory_storage

# Run with detailed output
icarus test --verbose --nocapture
```

### `icarus bridge`

Manage the bridge service that connects Claude Desktop to ICP canisters.

#### `icarus bridge start`

Start the bridge service.

**Usage:**
```bash
icarus bridge start --canister-id <id> [options]
```

**Arguments:**
- `--canister-id <id>` - Canister ID to connect to (required)

**Options:**
- `--ic-host <url>` - ICP network URL (default: https://ic0.app)
- `--log-level <level>` - Log level: error, warn, info, debug

**Examples:**
```bash
# Connect to local canister
icarus bridge start --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai

# Connect to mainnet canister
icarus bridge start --canister-id xkbqi-2qaaa-aaaah-qbpqq-cai --ic-host https://ic0.app
```

#### `icarus bridge status`

Check bridge service status.

**Usage:**
```bash
icarus bridge status
```

**Output:**
- Running/stopped status
- Connected canister ID
- Process ID
- Uptime

#### `icarus bridge stop`

Stop the running bridge service.

**Usage:**
```bash
icarus bridge stop
```

### `icarus connect`

Configure Claude Desktop to connect to your canister.

**Usage:**
```bash
icarus connect --canister-id <id> [options]
```

**Arguments:**
- `--canister-id <id>` - Canister ID to configure

**Options:**
- `--name <name>` - Custom name for the configuration
- `--output <file>` - Output configuration to file
- `--format <format>` - Output format: json, toml

**Examples:**
```bash
# Generate configuration
icarus connect --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai

# Save to file
icarus connect --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai --output claude-config.json

# Custom name
icarus connect --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai --name "My Memory Assistant"
```

### `icarus analyze`

Analyze a canister for MCP compatibility.

**Usage:**
```bash
icarus analyze [options]
```

**Options:**
- `--canister-id <id>` - Analyze deployed canister
- `--wasm <file>` - Analyze WASM file
- `--check-metadata` - Verify metadata endpoint
- `--check-tools` - List available tools

**Examples:**
```bash
# Analyze current project
icarus analyze

# Analyze deployed canister
icarus analyze --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai

# Check specific WASM file
icarus analyze --wasm ./my-canister.wasm
```

### `icarus generate`

Generate project files or components.

**Usage:**
```bash
icarus generate <type> [options]
```

**Types:**
- `tool` - Generate a new tool
- `type` - Generate a new data type
- `migration` - Generate a migration

**Options:**
- `--name <name>` - Name for generated item
- `--description <desc>` - Description for tools

**Examples:**
```bash
# Generate a new tool
icarus generate tool --name search --description "Search for items"

# Generate a new type
icarus generate type --name User

# Generate a migration
icarus generate migration --name add_user_email
```

### `icarus update`

Update the CLI to the latest version.

**Usage:**
```bash
icarus update [options]
```

**Options:**
- `--check` - Check for updates without installing
- `--force` - Force update even if already latest
- `--version <version>` - Update to specific version

**Examples:**
```bash
# Check for updates
icarus update --check

# Update to latest
icarus update

# Update to specific version
icarus update --version 0.2.0
```

### `icarus publish`

Publish your MCP server to the marketplace (coming soon).

**Usage:**
```bash
icarus publish [options]
```

**Options:**
- `--dry-run` - Validate without publishing
- `--category <category>` - Marketplace category
- `--price <price>` - Price in ICP tokens

**Status:** This command is planned for a future release.

## Configuration

The CLI stores configuration in `~/.icarus/config.toml`:

```toml
[auth]
session_token = "..."
expires_at = "2025-01-01T00:00:00Z"

[telemetry]
enabled = true
anonymous_id = "..."

[updates]
check_on_startup = true
last_check = "2025-01-01T00:00:00Z"

[preferences]
default_network = "local"
verbose_output = false
```

## Environment Variables

The CLI respects these environment variables:

- `ICARUS_CONFIG_DIR` - Override config directory location
- `ICARUS_LOG_LEVEL` - Set log level: error, warn, info, debug, trace
- `ICARUS_NO_COLOR` - Disable colored output
- `ICARUS_TELEMETRY` - Enable/disable telemetry: true, false
- `DFX_NETWORK` - Override default dfx network
- `IC_HOST` - Override default IC host

## Exit Codes

The CLI uses standard exit codes:

- `0` - Success
- `1` - General error
- `2` - Misuse of command
- `3` - Configuration error
- `4` - Network error
- `5` - Authentication error
- `126` - Command found but not executable
- `127` - Command not found

## Examples

### Complete Workflow

```bash
# 1. Create a new project
icarus new memory-assistant --with-tests

# 2. Navigate to project
cd memory-assistant

# 3. Run tests
icarus test --all

# 4. Deploy locally (builds automatically)
icarus deploy --network local

# 6. Start bridge (use canister ID from deploy output)
icarus bridge start --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai

# 7. Configure Claude Desktop
icarus connect --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai
```

### Development Workflow

```bash
# Use local SDK for development
icarus new test-project --local-sdk ~/projects/icarus-sdk

# Test specific functionality
icarus test --filter test_memory_operations

# Deploy with force to update
icarus deploy --force --network local
```

### Production Deployment

```bash
# Run all tests
icarus test --all

# Deploy to mainnet with adequate cycles
icarus deploy --network ic --cycles 5000000000000

# Set up monitoring
icarus analyze --canister-id <prod-id> --check-tools
```