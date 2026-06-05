## Why

`git vmr worktree add` can infer a shared branch name from the aggregate target path, but users cannot choose a different new branch name while creating the aggregate worktree. Git already supports this workflow with `git worktree add -b <new-branch> <path> [<commit-ish>]`, and VMR should expose the same short flag for aggregate worktrees.

## What Changes

- Add `-b <new-branch>` to `git vmr worktree add`.
- When `-b` is provided, create each child repository worktree on the requested new branch instead of inferring a branch name from the aggregate target path.
- Preserve optional `<commit-ish>` as the Git start point for the new branch when it is provided.
- Preserve existing behavior when `-b` is omitted.
- Do not add a `--branch` alias, `-B`, or other `git worktree add` flags in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: Add explicit `-b <new-branch>` branch creation behavior for `git vmr worktree add`.

## Impact

- CLI parsing for the nested `worktree add` command.
- Worktree command dispatch and branch selection logic.
- Existing Git worktree helper argument construction.
- Unit and integration tests for `git vmr worktree add`.
- Worktree command documentation and OpenSpec requirements.
