# Wayfinder map: diff command

Labels: wayfinder:map

## Destination

An implementation-ready spec for `git vmr diff` in `openspec/changes/diff-command/` — `spec.md` plus implementation tickets — ready to hand to a separate build effort. This map decides; it does not build.

## Notes

Settled during charting (constraints every ticket works within):

- **Surface**: worktree-vs-index by default, `--staged`, optional path filters routed to owning repos (aggregate path allowed). Commit ranges out of scope.
- **Output shape**: one monorepo-style combined diff with VMR-root-relative paths (`a/backend/src/x`), likely via git's own `--src-prefix`/`--dst-prefix`.
- **Colour**: delegated to git — `--color=always` to children when stdout is a TTY/pager, `--color=never` otherwise; `--color[=when]` flag mirrors git's.
- **Paging**: git-like — `GIT_PAGER` → `core.pager` → `PAGER` → `less` (FRX behaviour), TTY-gated, `--no-pager` supported.
- **Flags**: minimal v1 — only `--staged`, `--color`, `--no-pager`, and paths.
- **Failures**: repo outcomes — succeeding repos' diffs print; failures report through result aggregation; non-zero exit.

Skills: use `/grilling` and `/domain-modeling` for grilling tickets; `/codebase-design` for the pager-seam question.

## Decisions so far

<!-- one line per closed ticket -->

## Not yet specified

- Nothing substantial beyond the live tickets — the effort is compact. Revisit after the pager-seam decision in case it surfaces render-layer restructuring.

## Out of scope

- Commit ranges (`A..B`) — revisions don't align across independent repos.
- Output-shape flags (`--stat`, `--name-only`, `--name-status`, `-U<n>`, `-w`, …) — consciously deferred past v1; returns as a fresh effort if wanted.
- Implementing the command — the destination is the spec.
