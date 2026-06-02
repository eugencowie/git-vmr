## Why

`git vmr status` already tells users to run `git vmr restore` to unstage changes or discard working tree edits, but the command does not exist. Adding `restore` closes that workflow gap and completes the basic edit command set alongside `add`, `mv`, and `rm`.

## What Changes

- Add a `git vmr restore <pathspec>...` subcommand for discarding unstaged tracked changes in child repositories.
- Add `--staged` support for unstaging paths from child repository indexes.
- Add `--worktree` support, including combined `--staged --worktree` usage, matching Git restore's common restore targets.
- Route VMR-facing path arguments to immediate child Git repositories using the established lexical path model.
- Expand VMR-root `.` to all immediate child Git repositories, while rejecting invalid ownership such as `.gitvmr/`, non-Git children, files directly under the VMR root, and paths outside the VMR.
- Intentionally skip `--source` in this change.

## Capabilities

### New Capabilities
- `restore-command`: Defines `git vmr restore` behavior for restoring working tree and index state across child repositories from any working directory inside a virtual monorepo.

### Modified Capabilities

None.

## Impact

- CLI parsing and dispatch in `src/cli/mod.rs`.
- New restore command module under `src/cli/`.
- Existing routing helpers in `src/cli/routing.rs`, reused where practical.
- Integration tests covering restore behavior across one or more child repositories.
- No external runtime dependencies.
