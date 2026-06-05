## Why

`git vmr worktree remove` removes linked aggregate worktrees but leaves the branches created for those worktrees in every child repository. Users often create temporary aggregate worktrees for short-lived branches, so branch cleanup should be available as part of the same lifecycle operation.

## What Changes

- Add `-d` and `--delete` to `git vmr worktree remove`.
- When requested, remove each child worktree and then attempt to safely delete the branch that was checked out in that removed child worktree.
- Skip branch deletion for detached child worktrees.
- Keep branch deletion safe by using Git's normal branch deletion behavior, so unmerged branches or otherwise invalid deletions are reported by Git.
- Preserve best-effort behavior across child repositories and deterministic repository-suffixed result reporting.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: `git vmr worktree remove` gains optional safe branch deletion after successful child worktree removal.

## Impact

- CLI parsing for `git vmr worktree remove`.
- Worktree removal orchestration in `src/commands/worktree.rs`.
- Git worktree list/remove and branch deletion delegation in `src/git/*`.
- Worktree command integration tests and CLI parser tests.
- Worktree command documentation.
