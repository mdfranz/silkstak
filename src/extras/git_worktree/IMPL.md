# Git Worktree Module Implementation

The `git_worktree` module provides an integrated workflow for handling tasks in isolated git worktrees.

## Workflow

1. **Creation**: When triggered (e.g., via `/wt new` or a CLI flag), Zerostack creates a new git worktree in a temporary directory.
2. **Task Execution**: The agent works within this worktree, making changes and running tests in isolation from the main repository.
3. **Completion**: Once the task is done, the agent (or user) can trigger a merge.
4. **Merging & Cleanup**: Zerostack merges the worktree's branch back into the main branch, deletes the worktree directory, and removes the temporary branch.

## Benefits
- Prevents the agent from polluting the user's active workspace.
- Allows for easy rollbacks by just deleting the worktree.
- Enables parallel work on multiple features.
