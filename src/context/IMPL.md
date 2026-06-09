# Context Module Implementation

The `context` module handles the injection of external information into the agent's system prompt. This includes project-specific documentation and user-defined prompts/themes.

## Components

### `mod.rs`
- `ContextFiles` struct: Holds all loaded context data (agents, architecture, prompts, themes).
- `walk_context_files`: Recursively searches from the current directory up to the root for `AGENTS.md`, `CLAUDE.md`, and `ARCHITECTURE.md`.
- `load`: The main entry point to gather all context before starting an agent.

### `prompts/`
- Manages the library of "system prompts" (e.g., `code`, `architect`, `writer`).
- Loads embedded defaults and user-defined `.md` files from the config directory.
- `prompts/regen`: Allows the user to reset prompts to their default state.

### `themes/`
- Manages TUI color schemes.
- Similar to prompts, it loads embedded themes and allows for user overrides.
- `themes::apply`: Translates a theme definition into `renderer` color settings.

## Core Context Files
- **`AGENTS.md` / `CLAUDE.md`**: Used for project-specific instructions, conventions, and tool usage hints.
- **`ARCHITECTURE.md`**: Intended for high-level technical overviews of the codebase. Zerostack can automatically prompt the user to create this if it's missing.

## Usage
The `ContextFiles` are passed to the `agent::builder`, which extracts the relevant strings and appends them to the agent's system preamble.
