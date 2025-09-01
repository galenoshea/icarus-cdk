# icarus-core

Core abstractions for building MCP (Model Context Protocol) servers on the Internet Computer.

This crate provides the fundamental traits and types needed to build MCP servers that run as ICP canisters, including:

- Protocol types for MCP messages
- Error handling and result types
- Session management abstractions
- Tool and resource interfaces

## Usage

This crate is typically used as part of the main `icarus` SDK. Add it to your project:

```toml
[dependencies]
icarus = "0.1"
```

## License

Apache 2.0