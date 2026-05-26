## Why

Users familiar with `git rm` and the existing `git vmr rm` command naturally reach for `git vmr worktree rm` when removing aggregate worktrees. Supporting `rm` as a visible alias reduces command friction while preserving the existing `remove` spelling.

## What Changes

- Add `git vmr worktree rm` as a visible alias for `git vmr worktree remove`.
- Ensure the alias accepts the same arguments and flags as `remove`, including repeated `--force`/`-f`, `--delete`/`-d`, and `-D`.
- Ensure command help advertises `rm` as an alias for the remove subcommand.

## Capabilities

### New Capabilities

### Modified Capabilities
- `worktree-command`: Worktree remove gains a visible `rm` alias with equivalent parsing and behavior.

## Impact

- CLI parsing for the `worktree remove` subcommand.
- CLI help output for `git vmr worktree`.
- Parser and integration tests for worktree removal.
