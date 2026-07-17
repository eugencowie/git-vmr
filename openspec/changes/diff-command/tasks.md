## 1. CLI surface and TTY fact ([ticket 05](tickets/05-cli-surface-and-tty.md))

- [ ] 1.1 Add the `diff` subcommand to clap: `--staged` (alias `--cached`), `--color[=<when>]` (auto/always/never, bare = always), `--no-pager`, trailing paths; wire event metadata
- [ ] 1.2 Add the injected TTY bool to `CliContext` (production: `stdout().is_terminal()`; `for_tests`: false)
- [ ] 1.3 Create `src/commands/diff.rs` with `run(workspace, context)` resolving `--color=auto` against the TTY fact; unit-test the colour/pager gating decisions

## 2. Combined diff ([ticket 06](tickets/06-combined-diff.md))

- [ ] 2.1 Route paths to owning child repos (aggregate allowed, unowned errors) and invoke per repo via the Git runner: `diff [--staged] --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/ --color=<resolved> --no-ext-diff --binary [--] <pathspecs>`
- [ ] 2.2 Concatenate byte-faithfully in repo order; wire failures as repo outcomes (surviving diffs print, aggregation on stderr, non-zero exit via `render::fail`)
- [ ] 2.3 Unit tests: exact `assert_eq!` on full `Rendered.stdout` plus `fake.calls()` on each child's args; `contains`-style stderr assertions for failures

## 3. Rename/copy header rewrite ([ticket 07](tickets/07-rename-rewrite.md))

- [ ] 3.1 Implement the pure transform (extended-header regions incl. hunkless blocks; prepend `<repo>/`; quoted-path branch inside the quotes; coloured lines handled) and apply it per child before concatenation
- [ ] 3.2 Add the seven inline fixtures: rename with hunks, pure 100% rename, copy lines, `rename from` inside a hunk body, coloured variant, mixed multi-file patch, quoted-path rename (bytes from research/quoted-paths.md)

## 4. Pager at the emit choke point ([ticket 08](tickets/08-pager.md))

- [ ] 4.1 Add `pager: Option<String>` to `Rendered` (default None; existing commands unchanged)
- [ ] 4.2 Diff fills it: TTY and no `--no-pager` → resolve via `git var GIT_PAGER` from the VMR root through the runner seam (`cat`/empty → None)
- [ ] 4.3 Emit pages: spawn `sh -c` with `LESS=FRX`/`LV=-c` when unset, write stdout to pager stdin, wait, then print stderr; no re-print on pager death; direct-print fallback if `sh` cannot spawn
- [ ] 4.4 Tests: scripted fake answers `var GIT_PAGER`; emit tests use real `sh -c` recording scripts asserting verbatim stdin and stderr-after-exit ordering

## 5. Integration and the applyability contract ([ticket 09](tickets/09-integration-apply-round-trip.md))

- [ ] 5.1 `tests/diff.rs` via `assert_cmd`: worktree diff, `--staged`, path filters (owned/aggregate/unowned-error), one-failing-repo aggregation, uncoloured unpaged piped output
- [ ] 5.2 Apply round-trip: real edits plus a real rename (plus binary/mode-only/quoted-path if practical) → `git vmr diff` → `git apply -p1` onto a pristine VMR copy → trees match
