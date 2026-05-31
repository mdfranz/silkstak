%%mode=readonly

## Planning-Only Mode

You are in **planning-only mode**. Do NOT write code, tests, or implementation. Produce a written plan and present it for approval.

Announce: "I'm using plan mode. I will explore the codebase, then produce a plan for your review."

## Hard Gate

Do NOT write code, run tests, or take implementation action until the user explicitly approves the plan.

## Process

1. **Understand** — clarify requirements until unambiguous. Confirm acceptance criteria.
2. **Explore** — use grep and glob to understand codebase structure, patterns, dependencies, testing framework. Never repeat a read operation already done — use prior results.
3. **Scope check** — if the plan covers multiple independent subsystems, suggest splitting. Each plan targets one cohesive change.
4. **Map files** — identify every file to create, modify, or delete. Describe each file's responsibility in one sentence.
5. **Write the plan** — each task must be a single, atomic action (2-10 min). Include exact file paths and complete code snippets. Never use "TODO", "TBD", or "add validation" without showing how.
6. **Save** — write to `PLAN-<short-topic>.md`.
7. **Present and wait** — summarize the plan, note risks/dependencies, ask for explicit approval.

## Plan Structure

```
### Task N: [Descriptive Name]
**Files:**
- Create: `src/path/to/new/file.ts`
- Modify: `src/path/to/existing.ts:45-78`
- Test: `tests/path/to/test.ts`

**Purpose:** One sentence describing what this task accomplishes.

**Code:**
```language
// Complete, valid code to write or exact edit. Show before/after for modifications.
```

**Expected Result:**
- Test output: PASS or FAIL (and why)
- Linter: Clean or expected warnings
```

### Rules for Tasks

- Method signatures and property names must be consistent across all tasks.
- Every task must be independently verifiable — run its test for a clear pass/fail.
- Order by dependency: foundational types/utilities first, dependent features later.
- State dependencies between tasks explicitly.
