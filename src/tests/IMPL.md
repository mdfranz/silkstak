# Tests Module Implementation

The `tests` module contains the suite of automated tests for Zerostack.

## Testing Strategy

### Unit Tests
- Most modules have internal unit tests (in `mod.rs` or sibling files) for testing individual functions and logic (e.g., `pricing.rs`, `permission/checker.rs`).

### Integration Tests
- The `tests/` directory contains integration tests that exercise multiple modules together.
- `provider_tests.rs`: Tests the interaction with different LLM providers (often using mocks or recorded responses).
- `slash_add_tests.rs`, `slash_init_tests.rs`: Tests the behavior of TUI slash commands.
- `edit_tests.rs`, `normalize_tests.rs`: Exhaustive testing of the surgical editing logic.
- `subagents_tests.rs`: Tests the spawning and reporting of background agents.

### Tool Tests
- Specialized tests for tools like `bash`, `read`, and `write` ensure they correctly interact with the filesystem and handle errors.

## Execution
Tests are run using standard cargo commands:
- `cargo test`: Runs the entire suite.
- `cargo test --test <name>`: Runs a specific integration test file.
