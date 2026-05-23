## Why

VMR linked worktrees can be created across all child repositories, but users do not have a matching VMR-level command to remove those aggregate worktrees. Adding `git vmr worktree remove` closes the lifecycle gap and keeps worktree management aligned with Git's command shape.

## What Changes

- Add `git vmr worktree remove [-f|--force] <worktree>`.
- Resolve `<worktree>` as an aggregate worktree path and remove one child Git worktree per immediate child Git repository at `<worktree>/<repo-name>`.
- Support repeated force flags for Git parity, including `-f`, `--force`, `-ff`, and `--force --force`.
- Pass the requested force count through to each underlying `git worktree remove` invocation.
- Preserve best-effort fan-out behavior: attempt every child repository, report repository-suffixed Git failures, and leave successful removals in place when another child fails.
- Clean up the aggregate `.gitvmr` marker and empty aggregate directory only after all child removals succeed.

## Capabilities

### New Capabilities

### Modified Capabilities
- `worktree-command`: Add aggregate linked worktree removal, force flag forwarding, best-effort failure behavior, cleanup behavior, and CLI validation for the new `remove` subcommand.

## Impact

- CLI parsing gains a nested `worktree remove` subcommand with a counted `-f`/`--force` option.
- Command dispatch and Git delegation gain `git worktree remove` handling.
- Worktree command behavior expands to include aggregate cleanup after successful removal.
- Integration tests should cover clean removal, dirty-worktree failure, single and repeated force forwarding, partial failures, cleanup, `-C` path handling, and parser validation.
