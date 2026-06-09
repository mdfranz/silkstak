# Agent Module Implementation

The `agent` module is the core orchestration layer for AI interactions. It leverages the `rig` library to interface with LLM providers and manage tool-augmented conversations.

## Components

### `builder.rs`
- Responsible for constructing `rig::Agent` instances.
- Injects the system prompt (from `prompt.rs`), registers tools from the `tools/` module, and configures model parameters (temperature, max tokens).
- Handles the creation of both the main agent and specialized agents like the `/btw` side-question agent.

### `runner.rs`
- Manages the execution lifecycle of an agent run.
- `spawn_agent`: Starts a tokio task that wraps `rig`'s streaming chat.
- Translates `rig` events into Zerostack's internal `AgentEvent` stream.
- Implements the "continue" logic for multi-turn tool interactions (injecting "Please continue" prompts).
- Provides `run_print` for headless/CLI output and `run_subagent` for background tasks.

### `prompt.rs`
- Assembles the final system prompt sent to the LLM.
- Collects context from:
    - `SYSTEM_PROMPT` (hardcoded core instructions).
    - Current security mode and capabilities.
    - Loaded context files (`AGENTS.md`, `ARCHITECTURE.md`).
    - Active project memory.

### `tools/` (Subdirectory)
- Contains the implementation of all local capabilities available to the agent.
- Each tool is defined as a struct implementing `rig::completion::Tool`.
- See `agent/tools/IMPL.md` for details.

## Control Flow
1. UI calls `builder` to create an `AnyAgent`.
2. UI calls `runner::spawn_agent` with a user prompt and session history.
3. `runner` starts the LLM stream.
4. As the LLM calls tools, the `runner` executes them locally (subject to `permission` checks).
5. Tool results are fed back into the LLM stream.
6. The `runner` emits `AgentEvent`s (tokens, tool calls, results) to the UI.
