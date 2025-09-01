# Basic Memory Server Example

A simple MCP server that demonstrates core Icarus SDK features through a memory storage system.

## Features

This example implements a basic memory storage server with:
- **Persistent storage** using ICP's stable memory
- **CRUD operations** for managing memories
- **Tag-based organization** for categorizing memories
- **Search functionality** to find memories by tags

## Running the Example

### Prerequisites

- Rust 1.75+
- Icarus CLI (`cargo install icarus-cli`)

### Build and Deploy

1. **Build the canister:**
   ```bash
   cd examples/basic-memory
   icarus build
   ```

2. **Deploy locally:**
   ```bash
   # Start local ICP network
   dfx start --clean

   # Deploy the canister
   icarus deploy --network local
   ```

3. **Start the MCP bridge:**
   ```bash
   icarus bridge start --canister-id <your-canister-id>
   ```

4. **Configure Claude Desktop:**
   
   Add to your Claude Desktop MCP settings:
   ```json
   {
     "mcpServers": {
       "memory-server": {
         "command": "icarus",
         "args": ["bridge", "start", "--canister-id", "<your-canister-id>"]
       }
     }
   }
   ```

## Available Tools

The server provides the following MCP tools:

### `memorize`
Store a new memory with optional tags.
- **Parameters:**
  - `content` (string): The memory content to store
  - `tags` (optional array): Tags for categorization
- **Returns:** Memory ID

### `recall`
Retrieve a specific memory by its ID.
- **Parameters:**
  - `id` (string): The memory ID
- **Returns:** MemoryEntry object

### `list`
List all stored memories with optional limit.
- **Parameters:**
  - `limit` (optional number): Maximum number of memories to return
- **Returns:** Array of MemoryEntry objects

### `search_by_tag`
Search for memories containing a specific tag.
- **Parameters:**
  - `tag` (string): The tag to search for
- **Returns:** Array of matching MemoryEntry objects

### `forget`
Delete a memory by its ID.
- **Parameters:**
  - `id` (string): The memory ID to delete
- **Returns:** Boolean indicating success

### `count`
Get the total number of stored memories.
- **Returns:** Number of memories

### `clear_all`
Clear all memories (use with caution!).
- **Returns:** Number of memories cleared

## Code Structure

```
src/lib.rs
├── MemoryEntry struct      # Data model
├── stable_storage!         # Persistent storage declaration
├── generate_id()          # ID generation helper
└── tools module           # MCP tool implementations
    ├── memorize()
    ├── recall()
    ├── list()
    ├── search_by_tag()
    ├── forget()
    ├── count()
    └── clear_all()
```

## Key Concepts Demonstrated

1. **Stable Storage**: Using `stable_storage!` macro to declare persistent data
2. **Data Modeling**: Creating Candid-compatible data structures
3. **Tool Definition**: Using `#[icarus_tool]` to expose functions as MCP tools
4. **Query vs Update**: Understanding when to use `#[query]` vs `#[update]`
5. **Error Handling**: Returning `Result` types with meaningful error messages

## Testing

Run the tests:
```bash
cargo test
```

## Next Steps

- Add authentication to restrict access
- Implement memory search by content
- Add memory update functionality
- Create a web UI for browsing memories
- Add export/import capabilities