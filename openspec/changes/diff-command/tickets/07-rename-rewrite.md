# Rename/copy header rewrite transform

Type: implementation
Status: resolved
Blocked by: 06

## Task

Implement the pure, diff-local transform from [spec.md](../spec.md)
"Rename/copy header rewrite" and wire it in before concatenation.

- Region: extended headers of each file block — from `diff --git` to the
  first `@@`, or to the next `diff --git` / end of input when the block
  has no hunks (pure 100% renames — the common worktree case).
- Prepend `<repo>/` (no `a/`/`b/`) to the path on `rename from`,
  `rename to`, `copy from`, `copy to` lines.
- Quoted-path branch: when the token is C-quoted, the prefix goes inside
  the quotes after the opening `"`; no re-escaping. See
  [research/quoted-paths.md](../research/quoted-paths.md).
- Coloured lines: SGR wraps outside the text; the same rewrite applies.
- Seven inline fixtures (spec "Testing requirements" item 6; no fixtures
  directory — inline strings per repo convention). The quoted-path
  fixture's bytes are lifted verbatim from the research file, never
  invented.

This transform is what makes the applyability contract hold for renames —
`git apply -p1` from the VMR root rejects unrewritten rename patches.

## Comments

Implemented (2026-07-18): `rewrite_rename_headers` in
`src/commands/diff.rs` — a pure, line-oriented transform applied in
`Git::diff` before the child's output reaches concatenation. It tracks
the extended-header region with one bool (`diff --git` opens it, `@@`
closes it, so hunkless 100% renames stay open to the next block or end
of input) and prepends `<repo>/` on the four rename/copy lines: inside
the opening `"` for C-quoted tokens (no re-escaping, per the research),
at the token start otherwise. A small `after_leading_sgr` helper skips
leading SGR codes so coloured lines match the same prefixes. All seven
fixtures from the testing requirements are inline `assert_eq!` tests,
with the quoted-path bytes lifted verbatim from
[research/quoted-paths.md](../research/quoted-paths.md).
