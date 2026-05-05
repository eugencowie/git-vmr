## Why

Users can inspect changes across the virtual monorepo with `git vmr status`, but they still need to manually `cd` into each child repository to stage files. A VMR-level `add` command closes that loop by letting users stage paths from wherever they are working in the virtual monorepo.

## What Changes

- Add a `git vmr add <path>...` subcommand that stages files in the owning child repositories.
- Resolve path arguments relative to the effective working directory, including the global `-C <path>` behavior.
- Route each VMR-relative path to the immediate child Git repository that owns it, then invoke `git add` with repo-relative paths.
- Support paths that refer to files, directories, untracked files, modified files, and deleted files.
- Reject paths outside the virtual monorepo, paths inside `.gitvmr/`, and paths that do not map to a child Git repository.
- Treat `.` naturally: from the VMR root it stages all child repositories; from inside a child repo it stages that subtree.

## Capabilities

### New Capabilities
- `add-command`: Defines `git vmr add` behavior for staging files across child repositories from any working directory inside a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses VMR root discovery and the existing global `-C` working directory resolution.
- Introduces shared or command-local path routing from user-facing VMR paths to child repository paths.
- Adds integration tests covering staging from the VMR root, from inside child repositories, with `-C`, and across multiple repositories.
