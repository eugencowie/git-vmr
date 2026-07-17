# Testing strategy for diff, colour, and paging

Type: grilling
Status: resolved
Blocked by: 02

## Question

How is `diff` tested? The scripted fake covers git invocations, but this command adds surfaces the fake doesn't reach:

- TTY detection driving `--color` and pager activation — how is it injected/faked?
- Pager spawning — is the pager behind a seam with a fake adapter (mirroring the Git runner pattern), or exercised only in integration tests?
- Asserting the combined output (prefixed paths, repo ordering, failure aggregation) against scripted diffs.

Depends on where the pager seam lands ([02-pager-seam](02-pager-seam.md)).

Note: ticket 02's answer (decision 5) already proposes positions on the first two bullets — TTY as an injected bool on the CLI context, no pager trait (the pager command string is the seam; emit tests pass `cat` or a recording script), resolution via the scripted fake answering `var GIT_PAGER`. This ticket confirms or overturns them and settles the third bullet, plus fixtures for the rename-rewrite transform (decision 6).

## Answer

Grilled to shared understanding; ticket 02's decision 5 is **confirmed**, and
five further decisions settle the rest:

1. **TTY-true paths are unit-test-only.** The injected TTY bool on the CLI
   context (`for_tests` defaults it to `false`; tests flip it) covers the
   decision logic — `--color=always` to children, `pager: Some(...)` filled.
   No PTY harness dev-dependency, no test-only escape hatch in the production
   binary. Integration tests run through `assert_cmd`'s pipes and therefore
   live permanently in the non-TTY world — which is exactly what a piped
   consumer sees, so that's coverage, not a gap. The one line only a real
   terminal can exercise (`stdout().is_terminal()` itself) is a
   standard-library call not worth a harness.

2. **Decision 5 confirmed as-is**: no pager trait. Pager *resolution* is
   scripted-fake territory (the fake answers `var GIT_PAGER`); the pager
   command string itself is the spawn seam.

3. **Emit's pager tests spawn real `sh -c` recording scripts.** Tests pass
   e.g. `cat > <tempfile>` as the pager string and assert the pager's stdin
   received the buffered diff verbatim, and that stderr text prints only
   after the pager exits. Semantics are git-faithful: a pager that dies
   loses the diff — emit does **not** re-print on nonzero pager exit. The
   "print directly if the spawn fails" fallback covers only `sh` itself
   being unspawnable, which cannot be arranged portably; it stays as
   untested defensive code, not a tested contract.

4. **Combined-output unit assertions are exact, deliberately breaking the
   `contains` convention.** Diff's stdout is a byte-faithful concatenation
   whose fidelity is the product, so tests `assert_eq!` the full
   `Rendered.stdout` against the expected concatenation (repo ordering,
   separator-free joins, rewrite output). Two-sided with `fake.calls()`:
   recorded args prove each child got `--src-prefix=a/<repo>/`,
   `--dst-prefix=b/<repo>/`, the right `--color`, `--staged`, and routed
   pathspecs — the args side is what proves prefixing, since canned output
   only contains what the fixture says. Failure aggregation stays
   `contains`-style on stderr, matching how other commands assert outcomes:
   one child's git fails → surviving diffs still present in stdout, failure
   reported via result aggregation, non-zero exit.

5. **Integration tests include a `git apply -p1` round-trip.** Real edits
   plus a real rename across child repos → run `git vmr diff` through the
   real binary → apply captured stdout onto a pristine copy of the VMR →
   assert the trees match. It is the sharpest oracle available (prefix
   mistakes, rewrite corruption, ordering bugs — one assertion, no golden
   file). A test is not a promise: whether the spec *contractually* owns
   applyability remains [04-write-spec](04-write-spec.md)'s call; if 04
   declines, the test survives as an internal correctness check.

6. **Seven inline fixtures for the rename-rewrite transform** (no fixtures
   directory — repo convention is inline strings, and each patch is ~10
   lines): (1) rename with content hunks — headers rewritten, hunks
   untouched; (2) pure 100% rename with no `@@` at all — the region bound
   must mean "to the next `diff --git` or end", the case most likely to
   break a naive implementation and exactly what worktree diffs produce;
   (3) copy lines — kept as future-proofing, flagged: real invocations
   can't reach it since `copy from`/`copy to` only appear under
   `--find-copies-harder`, which we don't pass; (4) a hunk body containing
   a line beginning `rename from` — untouched, proving the region bound is
   real; (5) the coloured variant — SGR wraps rewritten identically;
   (6) multi-file patch where only some files are renames — per-block
   rewrite, neighbours untouched; (7) quoted-path rename, bytes lifted
   verbatim from the quoted-path research (below), never invented.

New research asset:
[research/quoted-paths.md](../research/quoted-paths.md) (git 2.53.0),
produced mid-grilling — it overturned the initial "leave quoted paths as
fog" position. Findings: `rename from`/`rename to` lines **are** C-quoted
when the path needs it (`"` always; non-ASCII under default
`core.quotePath=true`; spaces alone never); on the `diff --git` line the
quotes wrap the whole token *including* the prefix, so that rewrite is
`a/` → `a/<repo>/` inside the quotes; colour sits strictly outside the
quotes, so decision 6's SGR finding holds. The transform gains exactly one
branch — prepend `<repo>/` after the opening `"` when the token is quoted,
at path start otherwise; no re-escaping needed since the prefix contains no
escape-worthy bytes. Validated end-to-end with `git apply -p1`, including
non-ASCII renames.
