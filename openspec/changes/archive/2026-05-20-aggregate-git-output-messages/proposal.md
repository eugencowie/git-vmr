## Why

`git-vmr` currently appends repository names to individual Git output lines and prints one line per repository outcome. That keeps output traceable, but it becomes noisy in large virtual monorepos where dozens of repositories emit the same success or failure message.

## What Changes

- Add shared aggregation behavior for fan-out Git command results.
- Preserve Git's first meaningful output line for each repository outcome, but group identical messages before rendering.
- Render each unique success or failure message once with repository context attached as a grouped suffix.
- Avoid duplicate fatal prefixes when Git already reports a fatal error.
- Keep successful commands quiet when Git emits no output.
- Preserve best-effort fan-out behavior: all eligible repositories are attempted and any failure still makes the command exit non-zero.
- Do not add fuzzy or heuristic grouping of merely similar messages in this change.

## Capabilities

### New Capabilities
- `aggregate-command-output`: Shared rendering behavior for grouped Git command successes and failures across fan-out child repository operations.

### Modified Capabilities
None.

## Impact

- Affects CLI result collection and rendering in `src/cli.rs`.
- Affects Git command result shape in `src/git.rs` and command adapters under `src/git/`.
- Affects command integration tests that currently assert one line per repository.
- No new runtime dependencies are expected.
