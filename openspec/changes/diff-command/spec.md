# Spec: `git vmr diff`

Status: ready for implementation
Charted by: [map.md](map.md) — decisions live in the closed tickets it indexes.

## Summary

`git vmr diff` shows one monorepo-style combined diff across every child
repo in scope: worktree-vs-index by default, index-vs-HEAD with `--staged`,
optionally narrowed by path filters routed to the owning repos. Paths in the
output are VMR-root-relative (`a/backend/src/x`), the combined output pages
like git does, and — when piped — it is contractually an applyable patch.

## Command surface

```
git vmr diff [--staged] [--color[=<when>]] [--no-pager] [--] [<path>...]
```

- **Default**: worktree vs index, per child repo, in repo order.
- **`--staged`**: index vs HEAD instead. (`--cached` is accepted as an
  alias, mirroring git.)
- **`--color[=<when>]`**: `always` / `never` / `auto` (default `auto`),
  mirroring git's flag. Bare `--color` means `always`.
- **`--no-pager`**: print directly, never spawn a pager.
- **`<path>...`**: routed to owning child repos as repo-relative pathspecs.
  The aggregate path (the VMR root) is **allowed** and expands to all child
  repos. A path owned by no child repo is an error, matching other
  path-taking commands.

Out of scope for v1 (see the map's Out of scope): commit ranges (`A..B`)
and output-shape flags (`--stat`, `--name-only`, `-U<n>`, `-w`, …).

## Output contract

### Combined diff

Per child repo in scope, run through the Git runner seam:

```
git diff [--staged] --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/ \
    --color=<always|never> --no-ext-diff --binary [--] <pathspecs...>
```

- Trailing slash on both prefixes is mandatory (git concatenates verbatim).
- `--no-ext-diff` guards against `diff.external`; the CLI prefix flags
  already override all `diff.*` prefix config (verified, ticket 01).
- `--binary` so binary changes are representable in the patch.
- Rename detection stays **on** (no `--no-renames`) — see the rewrite below.

Child outputs are concatenated byte-faithfully in repo order with no
separators; repos with an empty diff contribute nothing. The combined text
is the command's `Rendered.stdout`.

### Rename/copy header rewrite

`rename from`/`rename to`/`copy from`/`copy to` lines are unprefixed by git
(patch-format design), so each child's output passes through a **pure,
diff-local transform** before concatenation (tickets 02 d6, 03 d6):

- Region: extended headers of each file block — from a `diff --git` line to
  the first `@@`, **or to the next `diff --git` / end of input** when the
  block has no hunks (pure 100% renames).
- Rewrite: prepend `<repo>/` (no `a/`/`b/`) to the path on those four lines.
- Quoted paths: when the path token is C-quoted, the prefix goes **inside**
  the quotes, immediately after the opening `"`; no re-escaping is needed
  (see [research/quoted-paths.md](research/quoted-paths.md)).
- Coloured variants: SGR codes wrap the whole line outside the quotes; the
  same transform applies to the text within.
- Lines inside hunk bodies that merely begin with `rename from` etc. are
  untouched (region bound is real).

### Applyability — a contract

**When stdout is not a TTY, the combined output is a valid patch that
`git apply -p1` (from the VMR root) accepts**, covering text, binary,
mode-only, symlink, and rename changes. This is a promise, not an
implementation accident: the integration suite round-trips it (ticket 03
d5), and any future change that breaks it is a breaking change. Colour is
the only applyability spoiler and is TTY-gated off exactly when the
promise applies.

## Colour and paging

One injected **TTY fact** (a bool carried on the CLI context) gates both,
so they can never disagree (ticket 02 d5):

- **Colour**: `--color=auto` resolves to `always` when stdout is a TTY,
  `never` otherwise; explicit `always`/`never` pass through. The resolved
  value is what children receive — the combined output is never
  re-coloured.
- **Paging**: active when stdout is a TTY and `--no-pager` is absent.
  Resolution is delegated to git: one `git var GIT_PAGER` invocation
  through the Git runner seam, run from the VMR root (resolves the full
  `GIT_PAGER` → `core.pager` → `PAGER` → `less` chain; a child repo's
  local `core.pager` cannot hijack the combined view). `cat` or an empty
  resolution means no paging.

### The pager seam

Paging is a property of the **emit choke point**, not the diff command
(ticket 02 d1–d3):

- `Rendered` gains `pager: Option<String>` — the shell command to page
  stdout through; `None` (the default, and every other command's value)
  means print directly. Interface symmetry is preserved: every command
  returns the identical shape.
- Commands stay pure and return the finished, buffered combined diff; emit
  pages it once (no streaming — that is a future performance change behind
  the same seam, not a spec change).
- Emit's order: spawn the pager via `sh -c`, exporting `LESS=FRX` and
  `LV=-c` when unset → write `stdout` into its stdin → `wait()` → print
  `stderr` → propagate the command result for the exit code. The pager
  sees only the diff; failures land on stderr **after** the pager exits.
- Git-faithful semantics: a pager that dies loses the diff — no re-print
  on nonzero pager exit. If `sh` itself cannot spawn, emit falls back to
  printing directly (untested defensive code, not a contract — ticket 03
  d3).
- Emit stays git-free, TTY-free, and format-blind.

## Failures

Repo outcomes, as elsewhere: succeeding repos' diffs still print in full;
each failing repo reports through result aggregation on stderr; the command
exits non-zero. With paging active, the aggregated failures print after the
pager exits.

## Testing requirements

From ticket 03 (all decisions confirmed):

1. TTY-true paths are **unit-only**: `for_tests` defaults the TTY bool to
   `false`; tests flip it to cover the decision logic. No PTY harness.
2. No pager trait: resolution is scripted-fake territory (`var GIT_PAGER`);
   the pager string itself is the seam.
3. Emit's pager tests spawn real `sh -c` recording scripts (e.g.
   `cat > <tempfile>`) asserting verbatim stdin delivery and
   stderr-after-exit ordering.
4. Combined-output unit tests use **exact `assert_eq!`** on the full
   `Rendered.stdout` (a deliberate break from the `contains` convention),
   two-sided with `fake.calls()` proving each child's args (prefixes,
   colour, `--staged`, routed pathspecs). Failure aggregation stays
   `contains`-style on stderr.
5. Integration includes a `git apply -p1` round-trip: real edits plus a
   real rename → `git vmr diff` via the real binary → apply onto a
   pristine VMR copy → trees match. This test enforces the applyability
   contract.
6. Seven inline fixtures for the rename rewrite (ticket 03 d6): rename
   with hunks; pure 100% rename (no `@@`); copy lines (future-proofing —
   unreachable without `--find-copies-harder`); `rename from` inside a
   hunk body untouched; coloured variant; multi-file mixed patch;
   quoted-path rename with bytes lifted verbatim from
   [research/quoted-paths.md](research/quoted-paths.md).

## Implementation tickets

Build order — each sized for one session:

1. [05-cli-surface-and-tty](tickets/05-cli-surface-and-tty.md)
2. [06-combined-diff](tickets/06-combined-diff.md)
3. [07-rename-rewrite](tickets/07-rename-rewrite.md)
4. [08-pager](tickets/08-pager.md)
5. [09-integration-apply-round-trip](tickets/09-integration-apply-round-trip.md)
