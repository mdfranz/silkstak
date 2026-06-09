# Extras Module Implementation

The `extras` module is a collection of optional or specialized features that extend Zerostack's core capabilities.

## Features

### `subagents/`
- Allows the agent to spawn background "sub-agents" to handle parallel tasks (e.g., "Explore these 5 directories and report back").
- Implements the `task` tool.

### `mcp/`
- Implements a client for the Model Context Protocol (MCP).
- Allows Zerostack to connect to external tool servers (e.g., Brave Search, Google Maps, local database explorers).
- Dynamically registers MCP tools with the main agent.

### `memory/`
- Provides project-specific persistent memory.
- Useful for storing long-term facts that shouldn't be in the main session history or `AGENTS.md`.

### `git_worktree/`
- Integration for managing tasks in isolated git worktrees.
- Automatically handles worktree creation, branch management, and cleanup/merging.

### `loop/`
- Implements the "Headless Loop" mode (`--loop`).
- The agent generates a `LOOP_PLAN.md`, executes it iteration by iteration, and validates results using a provided command.

### `acp/` (Agent Control Protocol)
- An experimental server mode that allows other applications to control Zerostack agents via a JSON-RPC-like interface.

### `archmd/`
- Helper logic for the interactive `ARCHITECTURE.md` creation flow.

### `status_signals/`
- Support for sending "busy/idle" signals to external listeners (e.g., a status bar widget) via Unix sockets.
