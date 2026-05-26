## Why

On Windows, `git vmr worktree list` can render aggregate worktree paths with mixed separators, for example `C:\Projects\vmr` beside `C:/Worktrees/new-feature`. Git-facing output should use Git-style forward slashes consistently, matching the path formatting already used by `git vmr status`.

## What Changes

- Normalize rendered aggregate worktree paths in `git vmr worktree list` to use `/` separators.
- Preserve existing grouping, filtering, sorting, branch, detached HEAD, and repository suffix behavior.
- Add coverage for Windows-style paths so separator normalization is enforced independent of the host platform.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `worktree-command`: `git vmr worktree list` output shall render aggregate worktree paths with Git-style `/` separators.

## Impact

- Affects worktree list output rendering in `src/commands/worktree.rs`.
- May introduce or reuse a small path display helper consistent with `git vmr status`.
- Adds focused unit and/or integration coverage for rendered path separators.
- No CLI arguments, exit statuses, Git invocations, or dependencies are expected to change.
