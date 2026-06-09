# ARCHMD Module Implementation

The `archmd` module provides specialized logic for the `ARCHITECTURE.md` file, which serves as a high-level map of the codebase for the agent.

## Features

### Template Creation
- If `ARCHITECTURE.md` is missing, Zerostack can automatically generate a template with sections for directory layout, module responsibilities, and key types.

### Agent Trigger
- When a new `ARCHITECTURE.md` is created, Zerostack can trigger a specialized agent run to "fill in the blanks" by exploring the codebase.

## Purpose
By maintaining a concise `ARCHITECTURE.md`, the user provides the agent with a "mental model" of the project that is more efficient than having the agent rediscover the structure in every session.
