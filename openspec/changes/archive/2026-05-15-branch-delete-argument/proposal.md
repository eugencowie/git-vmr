## Why

Users can already create a branch across all child repositories with `git vmr branch <branch-name>`, but deleting that branch still requires entering each repository and running `git branch -d` or `git branch -D` manually. Adding the deletion flags closes that workflow gap while preserving Git's own safety checks and force-delete behavior.

## What Changes

- Add `git vmr branch -d <branch-name>` to delete a local branch across discovered child repositories using Git's safe deletion semantics.
- Add `git vmr branch -D <branch-name>` to force-delete a local branch across discovered child repositories using Git's force deletion semantics.
- Keep deletion attempts independent and best-effort: failures in one repository do not prevent attempts in other repositories.
- Report per-repository deletion failures using the existing concise aggregate error style.
- Preserve existing branch listing and branch creation behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `branch-command`: Add safe and force branch deletion flags to the existing branch command behavior.

## Impact

- CLI argument parsing and validation for the `branch` subcommand.
- Branch command dispatch and per-repository Git command execution.
- Branch command integration tests and parsing tests.
- `branch-command` OpenSpec requirements.
