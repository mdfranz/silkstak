# UI Module Implementation

The `ui` module implements the interactive Terminal User Interface (TUI) for Zerostack. It uses `crossterm` for terminal manipulation and provides an asynchronous event-driven architecture.

## Architecture

### Main Loop (`mod.rs`)
- `run_interactive`: The entry point for the TUI.
- Manages the top-level state: current session, agent status, and active runners.
- Uses `tokio::select!` to multiplex between:
    - User input (keys, mouse, resize).
    - Agent events (from `agent_rx`).
    - Permission requests (from `ask_rx`).
    - Side-question events (from `btw_rx`).

### Rendering (`renderer.rs`)
- Implements a custom viewport-based renderer.
- Maintains a buffer of lines and handles scrolling, wrapping, and selection.
- `TerminalGuard`: Ensures the terminal is returned to a sane state (raw mode disabled, mouse capture off) on exit.

### Event Handling (`event_handler.rs`)
- Logic for processing `AgentEvent`s and updating the UI state.
- Handles markdown rendering of agent tokens as they arrive.
- Manages the transition between "thinking" and "idle" states.

### Subdirectories
- `input/`: The `InputEditor` component. Supports multi-line editing, history (`Ctrl-P`/`Ctrl-N`), and completions.
- `pickers/`: TUI overlays for selecting models, prompts, themes, and files.
- `slash/`: Handlers for commands starting with `/`. Implements features like `/provider`, `/model`, `/compress`, and `/session`.
- `events.rs`: UI-specific event definitions and rendering helpers (e.g., `render_session`).

## Design Patterns
- **Asynchronous Decoupling**: The agent runs in a separate tokio task, communicating with the UI via channels. This keeps the TUI responsive during long-running tool executions or slow LLM responses.
- **Stateless Rendering**: The `Renderer` is largely responsible for drawing the current state to the screen, while the main loop in `mod.rs` manages the transitions between states.
