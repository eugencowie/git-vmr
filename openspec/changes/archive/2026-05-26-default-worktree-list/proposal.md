## Why

`git vmr worktree` currently requires an explicit nested subcommand even though the most common read-only operation is `list`. Making `list` the default matches the user's expectation that the bare worktree command shows the worktree overview.

## What Changes

- Treat `git vmr worktree` with no nested subcommand as equivalent to `git vmr worktree list`.
- Preserve the existing `git vmr worktree list` output, validation, working directory behavior, and exit behavior.
- Keep unsupported worktree subcommands and invalid operands rejected during CLI parsing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: The bare `git vmr worktree` command defaults to the existing list behavior.

## Impact

- Affects CLI parsing and worktree command dispatch.
- Adds focused CLI and integration coverage for the default worktree command behavior.
- No dependency, storage, or output format changes.
