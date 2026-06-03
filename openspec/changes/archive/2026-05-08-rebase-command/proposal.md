## Why

Users working across a virtual monorepo can already inspect branches, create branches, commit, and merge across child repositories, but rebasing a shared branch still requires entering each repository and running Git manually. A VMR-level rebase command reduces that repetition while preserving each child repository's normal Git rebase behavior.

## What Changes

- Add a `git vmr rebase <upstream>` subcommand that attempts to rebase every immediate child Git repository onto the requested upstream.
- Treat rebase attempts as best-effort: a failure in one child repository SHALL NOT prevent rebase attempts in other discovered child repositories.
- Run child repository rebase attempts in parallel and report failures deterministically after all attempts complete.
- Succeed quietly when all rebases succeed, when there are no immediate child Git repositories, or when only non-Git child directories are present.
- Leave failed repositories in Git's normal post-rebase state, including an in-progress conflicted rebase when Git stops for conflicts.
- Reuse existing VMR root discovery and global `-C <path>` behavior.

## Capabilities

### New Capabilities

- `rebase-command`: Defines `git vmr rebase` behavior for best-effort parallel rebases across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses immediate child repository discovery conventions established by `branch`, `commit`, and `merge`.
- Shells out to Git for rebase semantics, consistent with existing command modules.
- Adds integration tests for clean rebases, conflicts, missing upstreams, non-Git children, quiet success, deterministic failure reporting, parallel best-effort behavior, and effective working directory handling.
