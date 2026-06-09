# Objective
Remove all token cost tracking functionality from the application to simplify the codebase and reduce complexity.

# Key Files & Context
- **State & Data Models**: `src/session/mod.rs`, `src/config/types.rs`
- **Pricing Logic**: `src/pricing.rs`
- **UI & Display**: `src/ui/mod.rs`, `src/ui/status.rs`, `src/ui/event_handler.rs`, `src/ui/slash/providers.rs`
- **Initialization & Config**: `src/main.rs`, `src/config/mod.rs`, `src/config/load.rs`, `src/provider.rs`
- **Tests & Docs**: `src/tests/btw_tests.rs`, `ARCHITECTURE.md`

# Implementation Steps
1.  **Delete `src/pricing.rs`**: Remove the file containing the `estimate_cost` calculation logic.
2.  **Update Data Models**:
    -   In `src/session/mod.rs`: Remove `total_cost`, `input_token_cost`, and `output_token_cost` fields from the `Session` struct and its `new` function.
    -   In `src/config/types.rs`: Remove `input_token_cost` and `output_token_cost` from the `QuickModel` struct.
3.  **Update Initialization & Configuration**:
    -   In `src/main.rs`: Remove code that resolves and assigns token costs to the session or prints them when listing models.
    -   In `src/config/load.rs`: Remove initialization of token costs for default quick models.
    -   In `src/config/mod.rs`: Update `save_quick_model` to remove the `input_cost` and `output_cost` parameters.
    -   In `src/provider.rs`: Remove cost information from the return type of `default_model_for_provider`.
4.  **Update UI and Event Handling**:
    -   In `src/ui/mod.rs`: Remove `btw_cost` and `btw_total_cost` variables from the UI event loop and `refresh_display` calls.
    -   In `src/ui/status.rs`: Remove the cost formatting logic and parameters from `StatusLine::render`.
    -   In `src/ui/event_handler.rs`: Remove the call to `estimate_cost` that updates the session's `total_cost`.
    -   In `src/ui/slash/providers.rs`: Remove cost arguments from the `/models-add` command parsing and display logic.
5.  **Update Tests & Docs**:
    -   In `src/tests/btw_tests.rs`: Remove assertions related to `session.total_cost`.
    -   In `ARCHITECTURE.md`: Remove the word "costs" from the `Session` description.

# Verification & Testing
-   Compile the project (`cargo check`) to ensure all references to removed fields and functions are resolved.
-   Run the test suite (`cargo test`) to confirm no tests are broken by the removal.
-   Run the application interactively to ensure the status bar renders correctly without the cost display.
-   Test the `/models-add` command to ensure it works without expecting cost parameters.