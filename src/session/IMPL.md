# Session Module Implementation

The `session` module manages the state of a single conversation, including its history, metadata, and persistence.

## Components

### `mod.rs`
- Defines the `Session` struct:
    - `messages`: A list of `SessionMessage` (role, content, estimated tokens).
    - `id`: A unique UUID for the session.
    - `model` / `provider`: The settings used for this session.
    - `total_cost`: Running tally of token costs.
    - `permission_allowlist`: Persistent "Allow Always" decisions made during the session.
- Implements `needs_compaction`: Determines when the context window is getting full.
- Implements `compress`: Logic for replacing old messages with a summary (compaction).

### `storage.rs`
- Handles saving and loading sessions from JSON files.
- Sessions are stored in the user's data directory (e.g., `~/.local/share/zerostack/sessions/`).
- Provides `find_recent_sessions` for the `--resume` feature.

### `chat_history.rs`
- Manages an append-only global history file of all user/assistant interactions across all sessions.
- Used for searching past interactions.

## Data Flow
1. On startup, a new `Session` is created or an existing one is loaded via `storage`.
2. As the user and agent talk, `add_message` is called to update the history.
3. The UI periodically calls `storage::save_session` to persist progress.
4. When `needs_compaction` returns true, the UI triggers the `/compress` logic, which updates the `messages` list in the `Session`.
