## Why

Users working across a virtual monorepo can inspect branches and create commits across child repositories, but merging a shared branch still requires entering each repository and running Git manually. A VMR-level merge command would make branch integration less repetitive while preserving each child repository's normal Git merge behavior.

## What Changes

- Add a `git vmr merge <commit-ish>` subcommand that attempts to merge the requested commit, branch, tag, or other Git revision in every immediate child Git repository.
- Treat merge attempts as best-effort: a failure in one child repository SHALL NOT prevent merge attempts in other discovered child repositories.
- Report successful merges and failed merges with repository names so users can see which repositories changed and which need attention.
- Leave failed repositories in Git's normal post-merge state, including conflicted merge state when Git reports conflicts.
- Reuse existing VMR root discovery and global `-C <path>` behavior.

## Capabilities

### New Capabilities
- `merge-command`: Defines `git vmr merge` behavior for best-effort merges across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses immediate child repository discovery conventions established by `branch` and `commit`.
- Shells out to Git for merge semantics, consistent with existing command modules.
- Adds integration tests for clean merges, conflicts, missing refs, non-Git children, deterministic failure reporting, and effective working directory handling.
