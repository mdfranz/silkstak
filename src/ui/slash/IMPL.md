# UI Slash Module Implementation

The `slash` module implements the handlers for all commands starting with `/`.

## Components

### `mod.rs`
- `handle_slash`: The main dispatcher that parses the command and routes it to the appropriate handler.
- `warm_model_cache`: Helper for fetching live model lists from providers to populate the picker.

### Handlers
- `providers.rs`: `/provider` and `/model` commands.
- `content.rs`: `/add` (adding file context) and `/clear` (removing context).
- `session.rs`: `/session` (listing/loading sessions) and `/new` (resetting).
- `settings.rs`: `/settings`, `/mode`, and `/theme`.
- `init.rs`: `/init` (project initialization flow).
- `memory.rs`: `/memory` (interacting with project memory).
- `features.rs`: `/reasoning`, `/tools`, and other toggleable features.
- `help.rs`: The `/help` command.

## Compaction Handler
- `handle_compress`: A complex handler that uses an LLM to summarize the conversation history when the context window is full, or when the user manually runs `/compress`.
