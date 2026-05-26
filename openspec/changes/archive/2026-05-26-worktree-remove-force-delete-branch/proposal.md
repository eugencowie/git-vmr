## Why

`git vmr worktree remove --delete` currently removes the associated branch only when Git considers safe deletion valid. Users need the same explicit force-delete escape hatch that `git vmr branch -D` already provides, without overloading the existing worktree `--force` flag.

## What Changes

- Add `git vmr worktree remove -D <worktree>` to remove the aggregate worktree and force-delete each successfully removed child worktree's checked-out local branch.
- Keep `-D` short-only; no long option is added.
- Keep `-f | --force` scoped to `git worktree remove`, so `--force --delete` remains safe branch deletion while `--force -D` combines forced worktree removal with forced branch deletion.
- Reject conflicting branch deletion modes such as `-d -D`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: Extend `git vmr worktree remove` branch deletion behavior with short-only forced branch deletion.

## Impact

- CLI parsing for `git vmr worktree remove`.
- Worktree removal command dispatch and branch deletion follow-up behavior.
- Focused CLI and integration tests for safe delete, force delete, conflicting flags, and rejected long form.
- `docs/git-vmr/worktree.md` option support table.
