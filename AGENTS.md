# Building

When compiling zerostack:
- Never run `cargo build`
- Don't use `--release` during development
- Never run `cargo check` (instead use `cargo test`)
- Always run `cargo fmt`
- Always run `cargo install --path . --debug`
- Run `cargo test` if you want to check all unit tests

# Testing

Important notes:
- Always write tests when writing new non-TUI code.
- Always update docs/ files when needed.
- If adding or editing slash commands, edit the slash commands `/` picker in the TUI.

DO NOT TEST unless I ask, especially when I ask you to review


# Documentation
- NEVER use file:/// references in Markdown. Use relative paths instead.
