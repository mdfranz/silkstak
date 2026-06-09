# UI Pickers Module Implementation

The `pickers` module provides the TUI components for interactive selection lists.

## Components

### `mod.rs`
- Defines the `Picker` trait and the `Picker` enum that wraps specific implementations.

### `list.rs`
- A generic searchable list picker. Used for selecting models, prompts, and themes.
- Supports fuzzy-like filtering as the user types.

### `file.rs`
- A specialized picker for browsing the filesystem. Used by the `/add` command.

### `models.rs`
- Data structures for representing model information in the picker.

### `handlers.rs`
- Logic for what happens when a picker selection is made (e.g., switching the provider or loading a prompt).
