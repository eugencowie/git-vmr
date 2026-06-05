## Why

`git vmr switch` currently switches child repositories only to branches that already exist. Users who create a coordinated feature branch across a virtual monorepo still need a separate branch creation step, even though Git's native `switch -c` / `switch --create` command already covers the common create-and-switch workflow.

## What Changes

- Add `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>`.
- Delegate create-and-switch behavior to Git in each child repository using `git switch --create <new-branch>`.
- Do not support optional start-point arguments for this change.
- Keep best-effort execution: attempt every discovered child Git repository, report failures, and return a non-zero status if any attempt fails.
- Deduplicate identical successful Git summary lines so repeated create-and-switch success output prints once without repository suffixes.
- Continue reporting failures with repository suffixes in deterministic order.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `switch-command`: Add create-and-switch forms, argument validation, deduplicated success output, and failure reporting behavior.

## Impact

- CLI parsing and dispatch in `src/cli.rs`.
- Switch command orchestration in `src/cli/switch.rs`.
- Git switch helper behavior in `src/git/switch.rs`.
- Switch command integration tests in `tests/switch.rs`.
- CLI parser tests and command documentation for `switch -c` / `switch --create`.
