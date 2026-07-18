# Per-repo diffs and the combined output

Type: implementation
Status: resolved
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

## Comments

Implemented (2026-07-18): `Workspace::map_routed` gathers per-repo values
through routing without reporting — the diff needs raw child output, not
grouped messages. `Git::diff` (in `src/commands/diff.rs`) builds the
spec'd invocation (`--src-prefix=a/<repo>/`, `--dst-prefix=b/<repo>/`,
resolved `--color`, `--no-ext-diff`, `--binary`, `--`, routed pathspecs)
and returns the raw diff paired with a `command_result` outcome
(`FailureReport::detailed("diff")`). `run` concatenates diffs in repo
order with no separators, then reuses `render::outcomes` for failure
aggregation, substituting the combined diff as `Rendered.stdout` (and
into the `Failed.rendered` via `render::fail` on failure, so surviving
diffs still flush). Unit tests use exact `assert_eq!` on the full stdout
plus `fake.calls()` arg assertions; failure aggregation and unowned-path
errors asserted `contains`-style.
