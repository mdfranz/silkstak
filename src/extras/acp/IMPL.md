# ACP Module Implementation

The `acp` (Agent Control Protocol) module implements a server that allows external applications to control Zerostack agents via a JSON-RPC-like interface.

## Design

- The ACP server listens for incoming connections (typically over a Unix socket or local TCP port).
- It provides methods for:
    - Sending prompts to an agent.
    - Receiving streamed tokens and tool calls.
    - Managing sessions.
    - Querying application state.

## Usage

ACP is used for integrating Zerostack into other tools, such as IDE plugins or custom automation scripts, providing a standardized way to leverage Zerostack's agent capabilities programmatically.
