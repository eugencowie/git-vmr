## Why

`git vmr worktree add` can create aggregate linked worktrees and `git vmr worktree remove` can clean them up, but users cannot move an aggregate linked worktree without dropping down into each child repository. Supporting `git vmr worktree move` fills that gap and keeps aggregate worktree management symmetric.

## What Changes

- Add `git vmr worktree move <worktree> <new-path>` to move every child linked worktree from one aggregate root to another.
- Add `-f` and `--force` support for `worktree move`, forwarding force occurrences to the underlying Git invocations.
- Preserve existing aggregate worktree behavior: resolve aggregate paths relative to the effective working directory, skip non-Git child directories, fan out across child repositories, and rely on Git for per-child safety checks.
- Maintain VMR discovery markers during aggregate moves, including leaving the destination marker in place when all child moves fail after destination setup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: Add aggregate linked worktree move behavior and force flag support.

## Impact

- CLI parsing for nested `worktree move` and its operands.
- Worktree command implementation and Git worktree delegation helpers.
- Integration and CLI parser tests for aggregate move behavior, force forwarding, marker handling, path resolution, and failure reporting.
- Worktree command documentation and support matrix.
