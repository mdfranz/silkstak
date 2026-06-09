# Permission Module Implementation

The `permission` module provides the security framework that gates all tool interactions. It ensures that the agent cannot perform destructive or unauthorized actions without user consent.

## Components

### `checker.rs`
- The `PermissionChecker` struct is the heart of the system.
- Evaluates tool calls against:
    - **Security Mode**: (Yolo, Standard, Guarded, ReadOnly, Restrictive).
    - **Glob Rules**: Patterns defined in `config.toml` (e.g., `permission.read = { "*" = "allow" }`).
    - **Regex Rules**: Strict denial patterns (e.g., blocking `rm -rf /`).
- Returns a `CheckResult`: `Allowed`, `Denied(reason)`, or `Ask`.

### `ask.rs`
- Implements the "Ask" flow for interactive confirmation.
- `AskRequest`: Sent from a tool to the UI when a permission check returns `Ask`.
- The UI displays a prompt to the user and sends back a `UserDecision` (Allow Once, Allow Always, Deny).

### `pattern.rs`
- Provides utility functions for glob and path matching.
- Handles the complexity of matching relative vs. absolute paths in tool arguments.

## Security Modes

- **Yolo**: All tool calls are allowed without confirmation.
- **Standard**: Allows safe read-only tools; asks for confirmation on write/edit/bash by default.
- **Guarded**: Asks for confirmation on almost everything.
- **ReadOnly**: Denies all write/edit/bash tools; allows read tools.
- **Restrictive**: A highly limited mode for untrusted environments.

## Integration
Tools call `check_perm` or `check_perm_path` in `agent/tools/mod.rs` before executing any action. These functions interface with the `PermissionChecker` (usually wrapped in an `Arc<Mutex<PermissionChecker>>` called `PermCheck`).
