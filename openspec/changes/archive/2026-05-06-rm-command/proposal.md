## Why

Users can stage additions and modifications across child repositories with `git vmr add`, but removing tracked files still requires manually entering each child repository or running raw Git commands with child-local paths. A VMR-level `rm` command completes the basic edit workflow by letting users remove tracked paths from wherever they are working in the virtual monorepo.

## What Changes

- Add a `git vmr rm <path>...` subcommand that routes paths to owning child repositories and invokes Git's `rm` behavior there.
- Resolve path arguments relative to the effective working directory, including the global `-C <path>` behavior.
- Route each VMR path to the immediate child Git repository that owns it, then invoke `git rm` with repo-relative paths.
- Support file removal and recursive directory/root removal only when `-r` or `--recursive` is supplied.
- Require `-r` or `--recursive` for `git vmr rm .` when it resolves to the VMR root, because that expands to recursive removal across all immediate child Git repositories.
- Reject paths outside the virtual monorepo, paths inside `.gitvmr/`, and paths that do not map to a child Git repository before invoking any Git removal.

## Capabilities

### New Capabilities
- `rm-command`: Defines `git vmr rm` behavior for removing tracked files across child repositories from any working directory inside a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses VMR root discovery and the existing global `-C` working directory resolution.
- Reuses or extracts the path routing model established by `git vmr add`.
- Adds integration tests covering removal from the VMR root, from inside child repositories, with `-C`, across multiple repositories, recursive directory/root behavior, and invalid-path preflight behavior.
