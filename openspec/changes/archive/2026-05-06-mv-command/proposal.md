## Why

Users can stage additions and removals across child repositories with `git vmr add` and `git vmr rm`, but moving tracked files still requires manually entering child repositories or composing filesystem and Git staging commands by hand. A VMR-level `mv` command completes the basic edit workflow by letting users move paths from wherever they are working in the virtual monorepo, including moves between different child repositories.

## What Changes

- Add a `git vmr mv <source> <destination>` subcommand for moving one tracked path within or between child Git repositories.
- Route both source and destination paths using the existing VMR ownership model and the effective working directory from cwd or `-C <path>`.
- Preserve normal `git mv` behavior for same-repository moves by delegating to Git inside the owning child repository.
- Support cross-repository moves by moving the filesystem path, then staging the source deletion and destination addition in their respective child repositories.
- Reject paths outside the VMR root, paths under `.gitvmr/`, explicit non-Git child paths, and root-level VMR files before moving anything.
- Keep the initial command focused on the two-operand move form; multi-source moves and advanced `git mv` flags are out of scope for this change.

## Capabilities

### New Capabilities
- `mv-command`: Defines `git vmr mv` behavior for moving paths within and across child repositories from any working directory inside a virtual monorepo.

### Modified Capabilities
- None.

## Impact

- Adds a new CLI subcommand and command module.
- Reuses or extends existing lexical path routing from `src/cli/routing.rs`.
- Adds integration tests for same-repository moves, cross-repository moves, effective working directory handling, destination-directory handling, Git failure reporting, and validation preflight behavior.
- Does not add external runtime dependencies.
