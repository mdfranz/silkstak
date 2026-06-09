# Subagents Module Implementation

The `subagents` module allows the main agent to delegate work to background agents, enabling parallel exploration and multi-step tasks.

## Components

### `mod.rs`
- Global configuration for sub-agents (shared client, default model).
- `init`: Sets up the sub-agent environment.
- `set_subagent_event_tx`: Allows sub-agents to report their tool calls back to the main UI.

### `task_tool.rs`
- Implements the `task` tool, which is exposed to the main agent.
- When called, it spawns a `subagent` runner.

### `builder.rs`
- Logic for building a `rig::Agent` specifically for sub-agent tasks.
- Typically uses a smaller/faster model (like Claude Haiku) and a specialized system prompt.

### `prompt.rs`
- The system prompt for sub-agents, emphasizing their role as focused assistants that report back to a "lead" agent.

## Execution
Sub-agents run their own tool execution loops, but their output is summarized and returned to the main agent as the result of the `task` tool call.
