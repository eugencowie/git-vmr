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

- [Validate --src-prefix/--dst-prefix for combined diffs](tickets/01-prefix-rewriting-research.md) — prefixes work everywhere except rename/copy lines; use `--no-renames --no-color --no-ext-diff --binary` per repo and the concatenated patch applies with `git apply -p1` from the VMR root (flags since git 1.5.4, CLI overrides all diff.* prefix config).

## Not yet specified

- Whether v1 promises an applyable patch at all: research shows applyability requires `--no-renames --no-color`, while the human view wants colour and (ideally) rename detection. The spec ticket must reconcile — e.g. colour+renames on TTY, apply-safe form when piped.
- Nothing else substantial beyond the live tickets — the effort is compact. Revisit after the pager-seam decision in case it surfaces render-layer restructuring.

## Out of scope

- Commit ranges (`A..B`) — revisions don't align across independent repos.
- Output-shape flags (`--stat`, `--name-only`, `--name-status`, `-U<n>`, `-w`, …) — consciously deferred past v1; returns as a fresh effort if wanted.
- Implementing the command — the destination is the spec.
