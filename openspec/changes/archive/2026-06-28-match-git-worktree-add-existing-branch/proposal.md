## Why

`git vmr worktree add <path>` currently always treats the inferred aggregate path basename as a new branch to create. Native `git worktree add <path>` instead checks out an existing branch with that basename when one exists, so VMR users hit a branch-exists error where Git would succeed.

## What Changes

- Change omitted-`<commit-ish>` `worktree add` behavior to infer the aggregate branch name from `<path>` and use it Git-style: check out the local branch when it already exists, otherwise create it from `HEAD`.
- Preserve aggregate VMR layout by continuing to derive the shared branch name from the aggregate target basename, not from child worktree path basenames.
- Keep explicit `-b <new-branch>` as create-only behavior that delegates branch-exists failures to Git.
- Keep explicit `<commit-ish>` behavior delegated to Git without creating or inferring an aggregate branch.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: `worktree add <path>` with omitted `<commit-ish>` should match Git shorthand behavior for existing local branches while preserving VMR aggregate branch inference.

## Impact

- Affects `src/commands/worktree.rs`, `src/git/worktree.rs`, and focused worktree integration tests.
- Updates `worktree-command` specs and user-facing worktree documentation.
- No CLI syntax, dependency, or storage format changes.
