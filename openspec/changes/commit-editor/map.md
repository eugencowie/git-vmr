# Wayfinder map: commit editor

Labels: wayfinder:map

## Destination

An implementation-ready spec in `openspec/changes/commit-editor/` — `spec.md` plus implementation tickets — for `git vmr commit` launching the user's git editor when `-m` is absent, with the resulting message committed in every dirty child repo. This map decides; it does not build.

## Notes

Settled during charting (constraints every ticket works within):

- **Approach**: implement the editor launch ourselves — resolve via `git var GIT_EDITOR` from the VMR root (mirroring the diff pager's `git var GIT_PAGER`), run the editor attached to the user's terminal, fan the message out to every dirty repo through the same commit path. Delegating to an interactive `git commit` in the first repo was rejected: its template shows one repo's status while others commit blind, it still needs a terminal-inheriting seam, and it makes "first repo" a load-bearing arbitrary choice.
- **Flags**: only `-m` becomes optional. All neighbour flags are out of scope.
- **Template**: the editor buffer should reflect the whole VMR (aggregate across dirty repos), not one repo — exact shape is the prototype ticket's question.

Skills: use `/grilling` and `/domain-modeling` for grilling tickets; `/codebase-design` for the seam ticket; `/prototype` for the template ticket.

## Decisions so far

<!-- one line per closed ticket -->

- [Research: git's editor and message-cleanup semantics](tickets/01-git-editor-semantics-research.md) — Git separates editor resolution, shell-aware execution, dynamic buffer presentation, and cleanup; unset/`dumb` `TERM` with no explicit editor is the special no-value case, not non-TTY stdin.
- [Prototype: the editor buffer template](tickets/02-template-prototype.md) — Use branch-grouped Git-style staged/unstaged sections with VMR-root-relative paths; omit identities, untracked files, and scissors presentation.

## Not yet specified

- Nothing further sighted — the space is small and the open questions are all ticketed.

## Out of scope

- Delegating to an interactive `git commit` in the first dirty repo and replaying its message — rejected during charting (see Notes).
- Neighbour commit flags (`-F`, `-e`/`--edit`, `--amend`, `--no-verify`, `--allow-empty-message`, …) — each has its own multi-repo semantics; returns as a fresh effort if wanted.
- Implementing the command — the destination is the spec.
