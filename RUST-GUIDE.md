# Rust Guide for Python and Go Developers

This guide explains the Rust implementation of `zerostack` using analogies from Python and Go. If you're familiar with those languages, this should help you navigate the codebase and understand the conventions used here.

## 1. Project Structure & Modules

In **Python**, you have packages and modules (folders with `__init__.py`). In **Go**, you have packages (folders).

In **Rust**, the structure is defined in `src/main.rs` or `src/lib.rs` using the `mod` keyword.

*   `mod agent;`: Tells Rust to look for `src/agent.rs` or `src/agent/mod.rs`.
*   `pub mod`: Makes the module accessible to other parts of the project (like `public` in Go).
*   `pub(crate)`: Visible only within this "crate" (project/library).

**Zerostack Example:**
Look at `src/main.rs` — it lists all the modules that make up the application:
```rust
mod agent;
mod auth;
mod cli;
// ...
```

---

## 2. Structs and Impls (Classes vs. Structs)

**Python**: You use `class` to group data and methods.
**Go**: You use `struct` for data and define methods on them.
**Rust**: Separate data (`struct`) from behavior (`impl`).

```rust
// Data
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
}

// Behavior
impl Session {
    pub fn new(provider: &str) -> Self {
        // ... constructor-like function
    }
}
```

---

## 3. Enums (The "Superpower")

**Python**: You might use `Enum` or `Union` types.
**Go**: You use interfaces or constants.
**Rust**: Enums can hold data. This is used extensively in `zerostack` to handle different LLM providers.

**Zerostack Example (`src/provider.rs`):**
Instead of a complex inheritance tree, we use `AnyAgent`:
```rust
pub enum AnyAgent {
    OpenAI(OpenAiAgent),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    Gemini(Agent<gemini::completion::CompletionModel>),
    // ...
}
```
This is like a **Type Union** in Python or an **Interface** in Go, but much more powerful because you can `match` on it.

---

## 4. Pattern Matching

**Python**: `match/case` (3.10+).
**Go**: `switch` (limited to values or type switches).
**Rust**: `match` is exhaustive. If you add a new provider to `AnyAgent`, the compiler will force you to update every `match` statement.

```rust
match agent {
    AnyAgent::OpenAI(a) => // ...
    AnyAgent::Anthropic(a) => // ...
}
```

---

## 5. Ownership, Borrowing, and Lifetimes

This is the biggest hurdle for Python/Go devs.

*   **Ownership**: Only one variable "owns" a piece of data at a time. When it goes out of scope, the memory is freed (no Garbage Collector needed!).
*   **Borrowing (`&`)**: Passing a reference. Like a pointer in Go, but the compiler ensures it never points to invalid memory.
*   **Mutable Borrowing (`&mut`)**: Only one mutable reference at a time. This prevents "Race Conditions" at compile time.

**Analogy:**
*   **Ownership**: Like giving someone your book. You don't have it anymore.
*   **Borrowing**: Like letting someone look at your book while you hold it.
*   **Mutable Borrowing**: Like giving someone your book to write in, but only one person can write at a time and you can't read it while they are writing.

---

## 6. Error Handling

**Python**: `try/except` (Exceptions).
**Go**: `if err != nil` (Explicit returns).
**Rust**: `Result<T, E>` enum. It's like Go, but you use the `?` operator to propagate errors upwards.

**Zerostack Example:**
We use `anyhow::Result<()>`. `anyhow` is a library that makes it easy to handle many different error types in one function (similar to `interface{}` error in Go).

```rust
fn load_config() -> anyhow::Result<Config> {
    let content = std::fs::read_to_string("config.json")?; // The '?' returns error if read fails
    let config = serde_json::from_str(&content)?;         // Returns error if JSON is bad
    Ok(config)
}
```

---

## 7. Option (No `null`/`nil`)

**Python/Go**: Variables can be `None` or `nil`.
**Rust**: No `null`. You must use `Option<T>`, which is either `Some(value)` or `None`.

### The "Box" Analogy
Think of `Option<T>` as a box that might be empty:
*   **`Some(value)`**: The box has something in it.
*   **`None`**: The box is empty.

This prevents the "Billion Dollar Mistake" (Null Pointer Exceptions) because you *must* handle the `None` case to get the value. You can't use the value inside a `Some` box without "unwrapping" it first.

**Zerostack Example:**
```rust
// A function that might not find a session
fn find_session(id: &str) -> Option<Session> {
    if let Some(s) = db.get(id) {
        Some(s)
    } else {
        None
    }
}

// Handling the result
match find_session("123") {
    Some(session) => println!("Found session: {}", session.id),
    None => println!("Session not found"),
}
```

---

## 8. Async/Await (Concurrency)

**Python**: `asyncio` (`async def`, `await`).
**Go**: Goroutines (`go func()`) and Channels.
**Rust**: Uses `tokio` (a library/runtime).

*   `#[tokio::main]`: The entry point for the async runtime.
*   `.await`: Suspends execution until the task is done (non-blocking).
*   **Channels**: Rust has them too (`tokio::sync::mpsc`). Used in `zerostack` for tool results and UI events.

---

## 9. Traits (Interfaces)

**Go**: Interfaces are satisfied implicitly.
**Rust**: Traits are satisfied explicitly (`impl Trait for Struct`).

In `zerostack`, we use traits from the `rig` library (like `CompletionModel`) to abstract away different LLM APIs.

---

## 10. Dependency Management

**Python**: `pip`, `requirements.txt`, `poetry`.
**Go**: `go mod`.
**Rust**: `Cargo`.

*   `Cargo.toml`: Like `package.json` or `go.mod`.
*   `Cargo.lock`: Like `go.sum` or `package-lock.json`. Ensures everyone uses the exact same versions.

---

## 11. Features and Conditional Compilation

**Go**: Build tags (`// +build ...`).
**Rust**: Features.

You'll see `#[cfg(feature = "subagents")]` throughout the code. This means that code is only compiled if the `subagents` feature is enabled in `Cargo.toml`. This is used to keep the binary small or to include experimental features.

---

## 12. Smart Pointers (Shared State)

**Go**: You just pass a pointer.
**Rust**: You must be explicit about how data is shared between threads.

*   `Arc<T>`: **A**tomic **R**eference **C**ounted. Allows multiple threads to "own" a piece of data (read-only).
*   `Mutex<T>`: Ensures only one thread can **mutate** the data at a time.
*   `Arc<Mutex<T>>`: The standard way to share mutable state across threads in Rust.

**Zerostack Example (`src/main.rs`):**
```rust
let perm: PermCheck = std::sync::Arc::new(std::sync::Mutex::new(checker));
```
This is how the `PermissionChecker` is shared between the main loop and the agent.

---

## 13. Project-Specific Types

*   `CompactString`: You'll see this instead of `String`. It's an optimization that stores small strings on the "stack" instead of the "heap," making the program faster and using less memory.
*   `anyhow::Result`: A flexible error type that can wrap any error.
*   `thiserror`: Used to define custom, strongly-typed errors (like `ToolError` in `src/agent/tools/mod.rs`).

---

## Quick Reference Table

| Concept | Python | Go | Rust |
| :--- | :--- | :--- | :--- |
| **Error Handling** | `try/except` | `if err != nil` | `Result` + `?` |
| **Nullability** | `None` | `nil` | `Option<T>` |
| **Concurrency** | `asyncio` | Goroutines | `tokio` (Async/Await) |
| **Interfaces** | ABCs / Protocols | Interfaces | Traits |
| **Packages** | Packages (`__init__.py`) | Packages (Folders) | Modules (`mod.rs`) |
| **Memory** | Garbage Collector | Garbage Collector | Ownership/Borrowing |
| **Data Types** | Classes | Structs | Structs + Enums |
