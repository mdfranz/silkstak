# Memory Module Implementation

The `memory` module provides project-specific persistent storage for facts, architectural decisions, and other long-term context.

## Design

- Memory is stored as simple markdown files or a small JSON database in the project's local state directory.
- Unlike session history, memory is intended to be durable across many different conversations.

## Integration

- At the start of every session, the memory is loaded and a summary is injected into the agent's system prompt.
- Agents can (optionally) be given tools to `memory_read`, `memory_write`, and `memory_search` to actively manage this context.
