## Why

Users can stage and inspect changes across child repositories with `git vmr add` and `git vmr status`, but committing still requires manually entering each child repository. A VMR-level commit command completes the basic edit workflow by creating coordinated commits in every child repository that has staged changes.

## What Changes

- Add a `git vmr commit -m <message>` / `git vmr commit --message <message>` subcommand.
- Discover immediate child Git repositories from the VMR root and attempt commits in every child repository that has staged changes.
- Skip non-Git child directories and child repositories with no staged changes.
- Report each successful commit as the first line of Git's commit output suffixed with the repository name, such as `[branch abcd123] Message (backend)`.
- Attempt all eligible repositories even when one commit fails, then return a non-zero exit status if any commit failed.
- Report each failed commit using the first non-empty Git stderr line suffixed with the repository name.
- Do not add `--allow-empty`, `-a`, `--all`, pathspec commit behavior, or editor-based message entry in this change.

## Capabilities

### New Capabilities
- `commit-command`: Defines `git vmr commit` behavior for committing staged changes across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses existing VMR root discovery and aggregate child repository scanning patterns.
- Uses the Git CLI for commit behavior and output compatibility.
- Adds integration tests for commit success, skipping, failure aggregation, message parsing, and working directory behavior.
