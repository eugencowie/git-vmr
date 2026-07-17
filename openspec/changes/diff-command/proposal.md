## Why

There is no way to see what changed across a VMR as one view — users must run
`git diff` per child repo and stitch the results mentally. A combined,
monorepo-style diff (with VMR-root-relative paths that even apply as a patch)
is the natural companion to the existing `status` command. The design was
fully charted by the wayfinder effort recorded in [map.md](map.md); this
proposal turns that charted spec into an implementable change.

## What Changes

- New `git vmr diff` workspace command: worktree-vs-index by default,
  `--staged`/`--cached` for index-vs-HEAD, path filters routed to owning
  child repos (aggregate path allowed), `--color[=<when>]`, `--no-pager`.
- Output is one combined diff with VMR-root-relative paths
  (`a/backend/src/x`) via `--src-prefix`/`--dst-prefix`, concatenated
  byte-faithfully in repo order.
- A pure, diff-local transform rewrites `rename from`/`rename to`
  (and `copy from`/`copy to`) headers with the repo prefix, including
  C-quoted paths, so rename patches stay applyable.
- **Contract**: when stdout is not a TTY, the output is a valid patch that
  `git apply -p1` accepts from the VMR root (text, binary, mode-only,
  symlink, and rename changes).
- Git-like paging at the emit choke point: `Rendered` gains
  `pager: Option<String>`; diff fills it (TTY-gated, resolved via
  `git var GIT_PAGER`); emit spawns `sh -c`, pages the buffered diff, and
  prints stderr after the pager exits. Other commands are unaffected
  (`None` default).
- Failures follow existing repo-outcome semantics: surviving diffs print,
  failures aggregate on stderr, non-zero exit.

Out of scope: commit ranges (`A..B`) and output-shape flags (`--stat`,
`--name-only`, `-U<n>`, `-w`, …).

## Capabilities

### New Capabilities

- `diff-command`: the `git vmr diff` command — surface, combined output
  format, rename-header rewrite, applyability contract, colour gating,
  and failure semantics.
- `output-paging`: the render-layer pager facility — the
  `Rendered.pager` field, pager resolution via git, emit's spawn/ordering
  semantics, and the direct-print fallback.

### Modified Capabilities

<!-- none — emit's behaviour for existing commands is unchanged (pager defaults to None) -->

## Impact

- `src/cli.rs` / `src/cli/context.rs`: new subcommand, injected TTY bool.
- `src/commands/diff.rs` (new): routing, per-repo invocation, rewrite,
  concatenation, colour/pager decisions.
- `src/render.rs`: `Rendered.pager` field and pager spawning in `emit`.
- `src/git.rs`: nothing structural — invocations go through the existing
  Git runner seam (`diff …`, `var GIT_PAGER`).
- `tests/diff.rs` (new): integration suite including the
  `git apply -p1` round-trip that enforces the applyability contract.
- No new dependencies; minimum git version unaffected
  (`--src-prefix`/`--dst-prefix` since git 1.5.4).

Detailed decisions live in the wayfinder artifacts: [spec.md](spec.md)
and tickets 01–09 under [tickets/](tickets/).
