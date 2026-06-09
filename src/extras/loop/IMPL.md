# Headless Loop Module Implementation

The `loop` module implements the autonomous execution mode for Zerostack.

## Components

### `mod.rs`
- `LoopState`: Tracks the current iteration, the plan, and validation results.

### `plan.rs`
- Logic for reading and updating `LOOP_PLAN.md`.
- The plan acts as the "source of truth" for the autonomous agent, describing the goal, the steps, and their status.

### `transcript.rs`
- Records a detailed log of every iteration in the loop, including the prompts, agent responses, tool outputs, and validation command results.

## Execution Flow
1. User provides a high-level goal and a validation command.
2. Agent creates a `LOOP_PLAN.md`.
3. Loop begins:
    - Agent reads the plan and its own last summary.
    - Agent performs work (using tools).
    - Validation command is run.
    - Results are recorded in the transcript.
    - Plan is updated with progress.
4. Loop continues until the goal is met or max iterations are reached.
