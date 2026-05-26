## Context

`git vmr worktree remove` is implemented as a Clap-derived subcommand under `WorktreeCommand::Remove`. The command already owns the complete removal behavior, including repeated force flags, branch deletion flags, dispatch, and integration tests.

This change only introduces an alternate spelling for the same subcommand. The alias must be visible so users can discover `rm` in command help.

## Goals / Non-Goals

**Goals:**
- Accept `git vmr worktree rm` anywhere `git vmr worktree remove` is accepted.
- Preserve identical parsing, dispatch, validation, and runtime behavior for all remove flags and operands.
- Advertise `rm` in `git vmr worktree --help`.

**Non-Goals:**
- Change worktree removal behavior.
- Add aliases for other worktree subcommands.
- Change the top-level `git vmr rm` command.

## Decisions

- Use Clap's visible subcommand alias support on `WorktreeCommand::Remove`.
  - Rationale: the alias is a pure parser/help concern, and Clap already owns subcommand parsing and help rendering.
  - Alternative considered: add a separate `Rm` enum variant and dispatch it to the remove implementation. That would duplicate the remove argument shape and increase the chance of future drift.
- Cover the alias with focused parser and integration tests.
  - Rationale: parser tests prove the alias maps to the same enum variant and accepts representative flags; an integration smoke test proves command dispatch reaches the existing remove path.

## Risks / Trade-offs

- Visible aliases affect help snapshots or help assertions if any are added later -> keep the alias intentionally visible and update help expectations when needed.
- Clap alias rendering may vary across Clap versions -> avoid brittle full-help assertions; test for the meaningful alias token when help coverage is needed.
