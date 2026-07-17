# Rename/copy header rewrite transform

Type: implementation
Status: open
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
