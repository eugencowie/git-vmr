## Why

`git vmr worktree` can create, remove, and move aggregate linked worktrees, but users cannot inspect the aggregate worktrees known across child repositories. Adding `git vmr worktree list` closes that workflow gap and helps users see the VMR-level worktree state before moving or removing worktrees.

## What Changes

- Add a nested `git vmr worktree list` command with no operands or options.
- List aggregate VMR worktrees discovered from each immediate child Git repository.
- Group child Git worktrees by aggregate parent path and show repository coverage when an aggregate worktree is only present in some child repositories.
- Show branch or detached-HEAD information for listed aggregate worktrees.
- Keep Git-compatible `worktree list` options such as `-v`, `--porcelain`, and `-z` unsupported in this change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `worktree-command`: Add aggregate worktree listing behavior to the existing worktree command capability.

## Impact

- CLI parsing in `src/commands.rs` and parser tests in `src/cli.rs`.
- Worktree command orchestration in `src/commands/worktree.rs`.
- Git worktree list parsing and data structures in `src/git/worktree.rs` and `src/git.rs`.
- Integration tests in `tests/worktree.rs`.
- Worktree command documentation in `docs/git-vmr/worktree.md`.
