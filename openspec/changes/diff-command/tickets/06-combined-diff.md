# Per-repo diffs and the combined output

Type: implementation
Status: open
Blocked by: 05

## Task

Produce the combined diff, per [spec.md](../spec.md) "Output contract"
(excluding the rename rewrite — ticket 07).

- Route `<path>...` through the workspace (aggregate path allowed,
  expanding to all child repos); unowned paths error like other
  path-taking commands.
- Per child repo, via the Git runner seam:
  `diff [--staged] --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/
  --color=<resolved> --no-ext-diff --binary [--] <pathspecs...>`.
  Trailing slashes on prefixes are mandatory. Renames stay on.
- Concatenate byte-faithfully in repo order, no separators; empty diffs
  contribute nothing.
- Failures as repo outcomes: surviving diffs still print, failures report
  through result aggregation on stderr, non-zero exit (use
  `render::fail` so stdout still flushes).
- Unit tests: exact `assert_eq!` on the full `Rendered.stdout`
  (deliberate break from the `contains` convention), plus `fake.calls()`
  asserting each child's args — prefixes, colour, `--staged`, routed
  pathspecs. Failure aggregation asserted `contains`-style on stderr.
