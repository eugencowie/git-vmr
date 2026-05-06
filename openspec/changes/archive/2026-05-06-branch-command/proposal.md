## Why

Users working in a virtual monorepo need a quick way to see which local branches exist across the child repositories. Today `git vmr status` shows checked-out branch state as part of working tree status, but it does not answer the branch inventory question without running `git branch` in each repository.

## What Changes

- Add a `git vmr branch` subcommand that lists local branches across all immediate child Git repositories in a VMR.
- Render output branch-centrically, grouping repositories by local branch name.
- Match Git's branch marker convention by prefixing active branch lines with `*` and inactive branch lines with a leading space.
- Omit repository names when a branch exists in every discovered child repository.
- Show repository names when a branch exists in only a subset of discovered child repositories.
- Render detached HEAD repositories as separate Git-like lines, for example `* (HEAD detached at 9773cf0) (frontend)`.

## Capabilities

### New Capabilities
- `branch-command`: Lists local branches across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses VMR root discovery and immediate-child repository scanning conventions established by `status`.
- Shells out to Git for branch and HEAD information, consistent with existing command modules.
- Adds integration and rendering tests for branch inventory output, detached HEAD handling, and repository-name elision.
