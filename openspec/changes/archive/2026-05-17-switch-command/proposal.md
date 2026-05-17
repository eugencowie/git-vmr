## Why

`git vmr` can already create, list, delete, merge, and rebase branches across child repositories, but switching every repository to a coordinated branch still requires manual per-repository commands or ad hoc shell loops. Adding `git vmr switch` closes that common branch workflow while preserving the project's existing best-effort, parallel execution model.

## What Changes

- Add a new `git vmr switch <branch-name>` command.
- Switch all discovered child Git repositories to the requested branch using Git's own `git switch <branch-name>` behavior.
- Run repository switch attempts in parallel.
- Treat the command as best-effort: attempt every repository, report all failures at the end, and return a non-zero exit status if any repository fails.
- Skip non-Git child directories and preserve existing VMR root discovery behavior.

## Capabilities

### New Capabilities
- `switch-command`: Defines `git vmr switch` behavior for switching branches across immediate child repositories in a virtual monorepo.

### Modified Capabilities
- None.

## Impact

- CLI parsing and dispatch in `src/cli.rs`.
- New switch command orchestration module under `src/cli/`.
- New Git switch helper under `src/git/`.
- Command support tables and generated command documentation.
- Integration tests for success, partial failure, deterministic error reporting, skipped non-Git children, and working directory discovery.
