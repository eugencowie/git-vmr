## Why

`git vmr worktree list` currently presents aggregate worktrees in a partially raw form: a single aggregate path can appear on multiple rows when child repositories differ by branch, detached state, or commit hash. This makes the command feel inconsistent with `git vmr branch` and `git vmr tag`, which both present a unified VMR-level view and use repository suffixes only to explain partial coverage.

## What Changes

- Change `git vmr worktree list` output so each aggregate worktree path is presented as one unified entry.
- Report shared branch state once for an aggregate worktree, omitting repository suffixes when all participating child repositories share that branch.
- Report mixed branch or detached states underneath the aggregate worktree entry with repository suffixes for the affected child repositories.
- Stop splitting branch-state output by child repository HEAD hash; branch names are the VMR-level state for branch-backed worktrees.
- Preserve existing aggregate discovery, filtering, CLI validation, and deterministic ordering behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: Change `git vmr worktree list` rendering from repeated aggregate-path rows to a unified aggregate worktree view aligned with `branch` and `tag` list output.

## Impact

- Affected code: `src/commands/worktree.rs` rendering logic.
- Affected tests: `tests/worktree.rs` list-output assertions plus focused renderer coverage if added.
- Affected docs: `docs/git-vmr/worktree.md` list-output description/examples if the project documents VMR-specific output separately from imported Git docs.
- No new dependencies or CLI options.
