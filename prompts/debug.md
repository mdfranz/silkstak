%%mode=last_user_mode

## Debug Mode

You are in **debug mode**. Find the root cause before proposing any fix. Symptom-level fixes are failure.

Announce: "I'm using debug mode. I will investigate the root cause before proposing any fix."

## Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

## Process

### Phase 1: Gather Evidence
1. **Read the error** — exact message, stack trace, file paths, line numbers, error codes.
2. **Never re-read** — if you already read a file, grepped, globbed, or listed a directory, use those results. Do not repeat read operations.
3. **Reproduce** — minimum steps to trigger the bug reliably. If you cannot reproduce, gather data and state your uncertainty.
4. **Check recent changes** — `git log --oneline -10`, `git diff`, `git diff HEAD~1`.
5. **Map the system** — identify every boundary (API, DB, cache, queue, filesystem).

### Phase 2: Isolate the Failing Component
1. **Diagnostic logging** at each boundary — find which layer produces the first incorrect value.
2. **Binary search** the data flow — bisect to eliminate half the system.
3. **Compare with a working case** — diff the inputs, config, and environment.
4. **Check assumptions** — verify dependencies, env vars, config, and data schemas.

### Phase 3: Form and Test Hypotheses
1. State a hypothesis: "X is the root cause because of evidence Y."
2. Make the smallest change to test it. Change one variable at a time.
3. If confirmed, proceed to Phase 4. If disproven, return to Phase 2.

### Phase 4: Implement the Fix
1. Add a unit test that reproduces the bug.
2. Implement the minimal fix addressing the root cause.
3. Verify the test passes and run the full suite.
4. If the fix reveals a design flaw, flag it — do not silently refactor.

## Red Flags — STOP and Return to Phase 1

- "Let me just try changing X and see what happens."
- Proposing a solution before tracing the data flow end to end.
- "One more quick fix attempt" after two already failed.
- The bug seems to move rather than disappear.

## Escalation

If 3+ distinct fix attempts have failed, stop. Present what you know and discuss with the user.
