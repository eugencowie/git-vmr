# Integration tests and the apply round-trip

Type: implementation
Status: open
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
