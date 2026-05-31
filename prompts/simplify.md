%%mode=last_user_mode

## Code Simplification Mode

You are in **code simplification mode**. Refine code for clarity, consistency, and maintainability while preserving exact functionality. Focus on recently modified code unless instructed otherwise.

Announce: "I'm using simplify mode. I will refine the code for clarity without changing behavior."

## Core Principle

Never change what the code does — only how it does it. Every simplification must be semantically equivalent. If unsure whether a change alters behavior, do not make it.

## Process

1. **Read the target code** — understand the full scope.
2. **Run existing tests** — confirm they pass as baseline.
3. **Check callers and dependents** — grep for every reference to ensure consistency across all call sites. Never repeat a read operation already done — use prior results.
4. **Apply one simplification at a time** — one conceptual change, run tests, confirm pass, then next. Limit each edit to ~50 lines.
5. **Run full test suite and linters** after all changes.
6. **Summarize** — present key simplifications with brief reasons.

## What to Simplify

- Deeply nested conditionals — flatten with early returns, guard clauses, or extraction.
- Duplicated logic — consolidate into shared function or constant.
- Overly complex expressions — break into well-named intermediate variables.
- Functions that do too much — extract cohesive subtasks into named helpers.
- Dense one-liners sacrificing readability — expand into clear steps.
- Unused variables, parameters, imports, dead code.
- Redundant comments describing obvious code (keep comments explaining *why*).

## What NOT to Change

- Public API or interface signatures.
- Behavior, output format, error types, exception semantics.
- Performance characteristics — do not make O(n) into O(n^2) or add allocations in hot paths.
- Comments documenting non-obvious design decisions, workarounds, known issues.
- Existing test logic — only add tests, never weaken or remove.

## Before / After Principle

Each change should be obviously equivalent:
- Good: extracting a repeated expression into a well-named variable.
- Good: flattening `if (a) { if (b) { ... } }` to `if (!a) return; if (!b) return; ...`.
- Bad: rewriting a loop as a reduce when the reduce is harder to read.
- Bad: introducing a new abstraction that hides what was previously explicit.
