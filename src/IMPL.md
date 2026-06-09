# Zerostack Implementation Overview

This document provides a high-level overview of the Zerostack codebase, its architecture, key interfaces, and module responsibilities.

## Architecture

Zerostack is an AI-assisted software engineering tool designed for both interactive (TUI) and headless usage. It leverages the `rig` library for agent orchestration and provides a rich set of local tools for codebase exploration and modification.

### Core Workflow

1.  **Startup**: `main.rs` parses CLI arguments (`cli.rs`) and loads configuration (`config/`).
2.  **Mode Dispatch**: Depending on flags, it either:
    - Runs a single prompt and exits (`--print`).
    - Enters a headless execution loop (`--loop`).
    - Starts the interactive TUI (`ui::run_interactive`).
3.  **Agent Orchestration**: The `agent` module (built on `rig`) manages the interaction with LLM providers (Anthropic, OpenAI, Gemini, Ollama).
4.  **Tool Execution**: Agents use tools defined in `agent/tools/` to interact with the local filesystem, run shell commands, etc.
5.  **Security**: All tool calls are gated by the `permission` system, which supports different safety modes (Yolo, Standard, Guarded, ReadOnly, etc.).
6.  **TUI Loop**: An asynchronous event loop (`ui/mod.rs`) handles user input, agent events (tokens, tool calls, results), and rendering.

## Directory Structure & Modules

### `/src` (Root)
- `main.rs`: Entry point and application wiring.
- `cli.rs`: CLI argument definitions using `clap`.
- `provider.rs`: Abstraction layer for LLM clients (OpenAI, Anthropic, Gemini, Ollama).
- `auth.rs`: API key resolution and authentication.
- `configure.rs`: Interactive configuration wizard.
- `pricing.rs`: Token cost estimation logic.

### `agent/`
- `mod.rs`: Module exports.
- `builder.rs`: Logic for constructing agents with specific prompts and toolsets.
- `runner.rs`: Execution loop for streaming agent responses and handling tool interactions.
- `prompt.rs`: System prompt generation, including context from context files.
- `tools/`: Individual tool implementations (bash, read, write, grep, edit, etc.).

### `ui/`
- `mod.rs`: Main interactive loop and event routing.
- `renderer.rs`: Custom terminal renderer using `crossterm`.
- `event_handler.rs`: Processes `AgentEvent` and `UserEvent` to update the state and UI.
- `markdown.rs`: Markdown-to-styled-text conversion for terminal display.
- `input/`: Input editor with history, multi-line support, and completions.
- `pickers/`: TUI components for selecting models, prompts, themes, and files.
- `slash/`: Handlers for slash commands (e.g., `/provider`, `/model`, `/compress`).

### `permission/`
- `checker.rs`: Logic for validating tool calls against glob/regex patterns and security modes.
- `ask.rs`: Asynchronous channel-based system for requesting user confirmation.
- `pattern.rs`: Glob and path matching logic.

### `session/`
- `mod.rs`: `Session` struct tracking messages, tokens, and cost.
- `storage.rs`: Persistent storage for session JSON files.
- `chat_history.rs`: Append-only global chat history.

### `context/`
- `mod.rs`: Loading and management of `AGENTS.md`, `ARCHITECTURE.md`, and custom prompt/theme files.
- `prompts/`: Management of the global and local prompt library.
- `themes/`: TUI color theme management.

### `extras/`
- `subagents/`: Support for delegating complex tasks to background agents.
- `mcp/`: Model Context Protocol client implementation.
- `memory/`: Persistent project-specific memory.
- `git_worktree/`: Integration for creating and managing git worktrees for tasks.
- `loop/`: Implementation of the headless agent loop mode.

## Key Interfaces & Data Flow

### `AgentEvent`
The primary communication channel from the agent runner to the UI.
```rust
pub enum AgentEvent {
    Token(CompactString),      // Text content
    Reasoning(CompactString),  // Model reasoning/thought
    ToolCall { name: CompactString, args: serde_json::Value },
    ToolResult { name: CompactString, output: CompactString },
    Done { response: CompactString, input_tokens: u64, output_tokens: u64 },
    Error(CompactString),
    // ...
}
```

### `Session`
Stores the state of the current conversation, including message history, total costs, and model/provider settings. Persisted to disk for resumption.

### `PermCheck` (Permission Checker)
A thread-safe guard that determines if a tool call is permitted based on the current `SecurityMode` and user-defined rules in `config.toml` or `GEMINI.md`.

## Design Principles

- **Surgical Edits**: Tools like `edit` prefer minimal changes to existing files to maintain codebase integrity.
- **Context Efficiency**: Automatic session compaction (`/compress`) and selective context loading from `AGENTS.md` / `ARCHITECTURE.md`.
- **Parallelism**: `/btw` side-questions and `subagents` allow for parallel exploration and work without blocking the main session.
## Subdirectory Layout Details

### `agent/`
- `builder.rs`: Implements `build_agent_inner` and `build_btw_agent_inner`, which configure the `rig::Agent` with the appropriate preamble, tools, and model.
- `runner.rs`: Contains the main execution logic. `spawn_agent` creates a tokio task that streams tokens and tool calls from the LLM, emitting `AgentEvent`s to a channel.
- `tools/`:
    - `bash.rs`: Executes shell commands in a persistent bash session or sandbox.
    - `read.rs` / `write.rs`: Direct filesystem access with safety checks.
    - `edit.rs`: Implements surgical file editing using various strategies (e.g., similarity-based matching).
    - `grep.rs` / `find_files.rs` / `list_dir.rs`: Optimized search and navigation tools.
    - `todo.rs`: Manages a session-local task list for the agent.

### `ui/`
- `renderer.rs`: A custom terminal UI engine. Manages a viewport, handles scrolling, and renders styled text.
- `event_handler.rs`: The bridge between the agent and the UI. It receives `AgentEvent`s and updates the `Session` and `Renderer` state.
- `input/`:
    - `mod.rs`: The `InputEditor` which handles multi-line input and command history.
    - `pickers.rs`: Logic for triggering and handling completion pickers.
- `slash/`:
    - `mod.rs`: Routing for `/` commands.
    - `add.rs`, `init.rs`, `providers.rs`, etc.: Individual command implementations.

### `permission/`
- `checker.rs`: The core `PermissionChecker` struct. It evaluates `CheckResult` (Allow, Deny, Ask) for tool/path combinations.
- `ask.rs`: Implements the interactive confirmation flow using `oneshot` channels to block tool execution until the user responds in the TUI.

### `extras/`
- `subagents/`:
    - `task_tool.rs`: The `task` tool which allows the main agent to delegate sub-tasks.
    - `builder.rs`: Configuration and building of sub-agent instances.
- `mcp/`:
    - `client.rs`: Implements the MCP client for communicating with external tool servers.
    - `tool.rs`: A generic tool wrapper that proxies calls to MCP servers.
- `memory/`:
    - `mod.rs`: Implements a simple key-value or document-based memory that is injected into the agent's context.
- `git_worktree/`:
    - `mod.rs`: Logic for `git worktree add`, `remove`, and automatic merging after task completion.
- `loop/`:
    - `plan.rs`: Logic for generating and following a `LOOP_PLAN.md` file in headless mode.
