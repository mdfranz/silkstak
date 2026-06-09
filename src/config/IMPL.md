# Config Module Implementation

The `config` module handles application-wide settings, model configurations, and tool limits.

## Components

### `mod.rs`
- Defines the main `Config` struct, which is serialized/deserialized from `config.toml`.
- Includes settings for:
    - Default model and provider.
    - Security and permission rules.
    - Tool-specific limits (max read lines, max grep results).
    - Subagent configurations.
    - MCP server definitions.

### `load.rs`
- Logic for finding and parsing the `config.toml` file.
- Usually located at `~/.config/zerostack/config.toml` or a platform-specific equivalent.
- Handles default value injection for missing fields.

### `types.rs`
- Supporting data structures for the configuration, such as `QuickModelConfig`, `CustomProviderConfig`, and `EditSystem` enums.

## Usage
The `Config` is loaded once at startup in `main.rs` and passed (often by reference or clone) to the UI, agents, and permission checkers to govern their behavior.
