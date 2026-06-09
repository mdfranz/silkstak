# Agent Tools Implementation

The `tools` module contains the set of local capabilities available to the agent. Each tool is a specialized struct that implements the `rig::completion::Tool` trait.

## Shared Infrastructure (`mod.rs`)

- **Permission Checking**: All tools must call `check_perm` or `check_perm_path` before execution.
- **Read Tracking**: Prevents the agent from repeatedly reading the same file section if it hasn't changed, saving tokens and preventing loops.
- **Edit System**: A global setting that determines whether `edit` uses similarity-based matching or other strategies.

## Core Tools

### `bash.rs` (`BashTool`)
- Executes shell commands.
- Supports a persistent bash session, allowing for stateful interactions (like `cd` and environment variables).
- Integrates with the `sandbox` system for isolated execution.

### `read.rs` (`ReadTool`)
- Reads file contents with support for line-based offsets and limits.
- Enforces the `max_read_lines` limit from the configuration.

### `write.rs` (`WriteTool`)
- Overwrites a file with new content.
- Automatically creates parent directories if they don't exist.

### `edit.rs` (`EditTool`)
- Performs surgical edits to existing files.
- Supports `similarity` mode (finding the best match for a block of text) and `exact` mode.
- Uses the `normalize` module to handle whitespace variations during matching.

### `grep.rs` (`GrepTool`)
- Performs recursive text search using `ripgrep` (if available) or a fallback implementation.
- Optimized for large codebases.

### `find_files.rs` (`FindFilesTool`)
- Searches for files by name/pattern using `fd` or a fallback.

### `list_dir.rs` (`ListDirTool`)
- Lists directory contents with filtering for common noise (e.g., `node_modules`, `target`).

### `todo.rs` (`WriteTodoList`)
- Manages a session-local task list. This is a "meta-tool" that helps the agent track its own progress across many turns.

### `crc.rs`
- Helper for calculating file CRCs, used by the `edit` tool to ensure it's working on the expected version of a file.
