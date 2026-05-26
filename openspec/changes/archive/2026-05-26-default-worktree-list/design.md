## Context

`git vmr worktree` is modeled as a top-level `Worktree` command with a nested `WorktreeCommand` enum. Today the nested command is required, so the bare command exits during CLI parsing instead of invoking the already-implemented list behavior.

The existing `WorktreeCommand::List` dispatch path already handles VMR root discovery, child repository scanning, output rendering, and errors. This change only needs to map the absent nested worktree command to that path.

## Goals / Non-Goals

**Goals:**

- Make `git vmr worktree` behave exactly like `git vmr worktree list`.
- Preserve all existing `git vmr worktree list` output and validation semantics.
- Keep the change localized to CLI parsing/dispatch and focused tests.

**Non-Goals:**

- Add new worktree list options or output formats.
- Change unsupported worktree subcommand handling.
- Change worktree aggregation, sorting, path normalization, or Git invocation behavior.

## Decisions

- Represent the nested worktree subcommand as optional at the CLI boundary and resolve `None` to `WorktreeCommand::List` during command dispatch.
  - Rationale: this keeps the defaulting rule near the worktree command and reuses the existing list implementation.
  - Alternative considered: add a separate `Default` enum variant. That would add another internal state even though runtime behavior is identical to `List`.

- Add both parser-level and integration coverage.
  - Rationale: parser coverage proves the CLI shape maps to the intended command, while integration coverage proves stdout, stderr, and exit status match the explicit list command.
  - Alternative considered: only add parser coverage. That would miss regressions where dispatch no longer routes to the same list implementation.

## Risks / Trade-offs

- Bare `git vmr worktree` will no longer show a missing-subcommand parse error -> This is intentional and covered by the new requirement.
- Optional nested subcommand parsing could accidentally loosen invalid input handling -> Preserve existing rejection tests for unsupported subcommands and invalid `list` operands/options.
