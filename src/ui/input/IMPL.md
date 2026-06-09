# UI Input Module Implementation

The `input` module implements a sophisticated terminal input editor capable of handling multi-line text, command history, and context-aware completions.

## Components

### `mod.rs` (`InputEditor`)
- The main struct managing the input state (buffer, cursor position, history).
- Handles raw `KeyEvent`s from `crossterm`.
- Implements `Ctrl-P` / `Ctrl-N` for history navigation.
- Implements `Ctrl-G` to open the current buffer in an external editor (e.g., `vim`).
- Manages the active "picker" (completion UI).

### `cursor.rs`
- Logic for calculating cursor screen coordinates based on terminal width and multi-line wrapping.

### `pickers.rs`
- Logic for triggering and filtering various TUI pickers (prompts, themes, models) while typing (e.g., when the user types `/` or `.`).

## Workflow
1. The TUI loop in `ui/mod.rs` passes key events to `InputEditor::handle_key`.
2. `InputEditor` updates its internal buffer and cursor.
3. If a special character is typed (like `/`), a picker is opened.
4. When the user presses `Enter` on a non-empty buffer, the text is returned to the TUI loop for processing.
