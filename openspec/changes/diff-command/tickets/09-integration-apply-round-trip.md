# Integration tests and the apply round-trip

Type: implementation
Status: resolved
Blocked by: 06, 07, 08

## Task

End-to-end coverage in `tests/diff.rs` via `assert_cmd` (permanently
non-TTY — exactly the piped-consumer view), per [spec.md](../spec.md)
"Testing requirements" item 5.

- Basic flows: worktree diff, `--staged`, path filters (owned, aggregate,
  unowned-error), one-failing-repo aggregation with non-zero exit,
  `--no-pager`/non-TTY producing uncoloured unpaged output.
- The applyability round-trip enforcing the spec's contract: real edits
  plus a real cross-file rename in child repos → run `git vmr diff`
  through the real binary → `git apply -p1` the captured stdout onto a
  pristine copy of the VMR → assert the trees match. One assertion, no
  golden file; catches prefix mistakes, rewrite corruption, and ordering
  bugs.
- Include binary, mode-only, and quoted-path (non-ASCII) changes in the
  round-trip tree if practical.

## Comments

Implemented (2026-07-18): `tests/diff.rs` covers the basic flows —
worktree diff across repos, `--staged` (index-only), an owned path routed
to one repo, the aggregate `.` path, an unowned-path error, one-failing-
repo aggregation (surviving diff on stdout, `diff failed` on stderr,
non-zero exit), and piped/`--no-pager` output asserted free of SGR escapes
with `GIT_PAGER` poisoned to prove no pager is consulted. The round-trip
stages a text edit, a pure cross-directory rename, a quoted-path
(non-ASCII) rename, a binary change, and a mode-only chmod across two
repos, snapshots a pristine copy first, captures `git vmr diff --staged`
through the real binary, applies it with `git apply -p1` from the pristine
VMR root, and asserts the trees match (file sets, bytes, executable bit;
`.git` excluded).
