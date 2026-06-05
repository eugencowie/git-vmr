## Why

`git reset` is currently listed as unsupported, leaving users without an aggregate way to move child repository HEADs, indexes, or working trees to a known state. Adding a focused reset command fills a core porcelain gap and matches the existing VMR model for repository-wide operations such as merge, rebase, fetch, pull, and push.

## What Changes

- Add `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]`.
- Run reset attempts across all immediate child Git repositories discovered from the VMR root.
- Execute child repository resets in parallel while reporting aggregate results deterministically.
- Treat reset as best-effort: failures in one child repository do not prevent attempts in other child repositories, and successful resets are not rolled back.
- Delegate mode semantics, commit resolution, conflict handling, and dirty working tree behavior to Git in each child repository.
- Keep pathspec reset forms out of scope for this change.

## Capabilities

### New Capabilities
- `reset-command`: Defines aggregate `git vmr reset` behavior across child Git repositories, including supported reset modes, optional commit handling, best-effort parallel execution, deterministic reporting, and VMR working directory behavior.

### Modified Capabilities

## Impact

- CLI parsing in `src/cli.rs` for the new `reset` subcommand and mutually exclusive mode flags.
- New command orchestration module under `src/cli/`.
- New Git invocation helper under `src/git/`.
- Integration tests for reset behavior, errors, parsing, and `-C` discovery.
- Command support documentation and command matrix updates.
