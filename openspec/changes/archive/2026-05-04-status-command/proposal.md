## Why

Users working with a virtual monorepo need to see the status of all constituent git repositories at a glance. Today they must `cd` into each repo and run `git status` individually. A unified status command shows the full picture in one invocation, grouped by branch so that branch divergence across repos is immediately visible.

## What Changes

- Add a `git vmr status` subcommand that traverses upward from the working directory to find the `.gitvmr/` marker, then reports the combined git status of all immediate child git repositories under the VMR root
- Output is a single unified status grouped by branch name — when all repos share a branch, the output is indistinguishable from regular `git status`; when branches diverge, separate sections make the mismatch obvious
- Paths in the output are relative to the current working directory, so running from different points in the tree produces naturally adjusted paths
- Status collection runs in parallel across repositories for performance
- File entries are color-coded (green for staged, red for unstaged and untracked), matching git's conventions. Headers and branch names remain uncolored.

## Capabilities

### New Capabilities
- `vmr-root-discovery`: Traverse directory ancestors to locate the `.gitvmr/` marker and resolve the VMR root directory
- `status-command`: Collect and render unified git status across all immediate child repositories of the VMR root, grouped by branch name, with cwd-relative paths

### Modified Capabilities

## Impact

- New CLI subcommand (`status`) added to the command set
- New dependency on a git library for repository introspection
- No breaking changes to existing commands
