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
- [Where does the pager live?](tickets/02-pager-seam.md) — at the emit choke point: `Rendered` gains `pager: Option<String>`, the diff command fills it (TTY-gated, resolved via `git var GIT_PAGER` through the runner seam), emit spawns `sh -c` and pages the buffered diff, stderr lands after the pager exits. Also: rename/copy lines are fixed by a diff-local pure transform (prepend `<repo>/`), verified applyable with `git apply -p1` — the `--no-renames` workaround from ticket 01 is dissolved; renames stay on.
- [Testing strategy for diff, colour, and paging](tickets/03-testing-strategy.md) — decision 5 confirmed; TTY-true paths are unit-only (injected bool, no PTY harness); emit's pager tests spawn real `sh -c` recording scripts with git-faithful no-re-print semantics; combined stdout asserted with exact `assert_eq!` (deliberate break from the `contains` convention) plus `fake.calls()` for prefix/colour/pathspec args; integration adds a `git apply -p1` round-trip; seven inline fixtures for the rename rewrite, including quoted paths per new research ([research/quoted-paths.md](research/quoted-paths.md) — rename from/to lines are C-quoted, prefix lands inside the quotes, transform gains one prepend-after-quote branch).

## Not yet specified

- Nothing — the way to the destination is clear once the last live ticket ([04-write-spec](tickets/04-write-spec.md)) resolves.

## Out of scope

- Commit ranges (`A..B`) — revisions don't align across independent repos.
- Output-shape flags (`--stat`, `--name-only`, `--name-status`, `-U<n>`, `-w`, …) — consciously deferred past v1; returns as a fresh effort if wanted.
- Implementing the command — the destination is the spec.
