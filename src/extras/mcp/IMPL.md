# MCP Module Implementation

The `mcp` module integrates the Model Context Protocol (MCP) into Zerostack, allowing it to use tools provided by external servers.

## Components

### `client.rs`
- Implements the MCP client logic.
- Connects to servers via stdio (running a command) or HTTP/SSE.
- Handles the JSON-RPC lifecycle of the protocol.

### `tool.rs`
- `McpTool`: A bridge that implements Zerostack's tool interface and proxies calls to the MCP server.

### `config.rs`
- Definitions for `McpServerConfig` in `config.toml`.
- Supports both local command-based servers and remote URL-based servers.

## Workflow
1. At startup (or on first use), Zerostack connects to all configured MCP servers.
2. It discovers the tools exposed by each server.
3. These tools are dynamically added to the agent's toolset.
4. When the agent calls an MCP tool, Zerostack sends the request to the external server and returns the result.
