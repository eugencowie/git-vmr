## Why

Users working across a virtual monorepo often need to create the same topic branch in every child repository before starting coordinated work. Today they must run `git branch <name>` separately in each repository, which is repetitive and easy to miss.

## What Changes

- Add an optional `[branch-name]` positional argument to `git vmr branch`.
- Preserve current `git vmr branch` behavior when no branch name is provided: list local branches across child repositories.
- When a branch name is provided, attempt to create that branch in every immediate child Git repository.
- Run branch creation independently per repository so one repository failure does not prevent attempts in other repositories.
- Report any failed repository branch creations after all attempts complete, using concise Git-like error lines annotated with the repository name.

## Capabilities

### New Capabilities

### Modified Capabilities

- `branch-command`: Add optional branch creation behavior and failure reporting semantics for `git vmr branch <branch-name>`.

## Impact

- CLI parsing for the `branch` subcommand.
- `src/cli/branch.rs` behavior for dispatching list mode versus create mode.
- Branch command integration tests and rendering/error handling tests.
- `branch-command` OpenSpec requirements.
