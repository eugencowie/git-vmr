## 1. CLI surface and TTY fact ([ticket 05](tickets/05-cli-surface-and-tty.md))

- [x] 1.1 Add the `diff` subcommand to clap: `--staged` (alias `--cached`), `--color[=<when>]` (auto/always/never, bare = always), `--no-pager`, trailing paths; wire event metadata
- [x] 1.2 Add the injected TTY bool to `CliContext` (production: `stdout().is_terminal()`; `for_tests`: false)
- [x] 1.3 Create `src/commands/diff.rs` with `run(workspace, context)` resolving `--color=auto` against the TTY fact; unit-test the colour/pager gating decisions

## 2. Combined diff ([ticket 06](tickets/06-combined-diff.md))

- [x] 2.1 Route paths to owning child repos (aggregate allowed, unowned errors) and invoke per repo via the Git runner: `diff [--staged] --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/ --color=<resolved> --no-ext-diff --binary [--] <pathspecs>`
- [x] 2.2 Concatenate byte-faithfully in repo order; wire failures as repo outcomes (surviving diffs print, aggregation on stderr, non-zero exit via `render::fail`)
- [x] 2.3 Unit tests: exact `assert_eq!` on full `Rendered.stdout` plus `fake.calls()` on each child's args; `contains`-style stderr assertions for failures

## 3. Rename/copy header rewrite ([ticket 07](tickets/07-rename-rewrite.md))

- [x] 3.1 Implement the pure transform (extended-header regions incl. hunkless blocks; prepend `<repo>/`; quoted-path branch inside the quotes; coloured lines handled) and apply it per child before concatenation
- [x] 3.2 Add the seven inline fixtures: rename with hunks, pure 100% rename, copy lines, `rename from` inside a hunk body, coloured variant, mixed multi-file patch, quoted-path rename (bytes from research/quoted-paths.md)

## 4. Pager at the emit choke point ([ticket 08](tickets/08-pager.md))

- [x] 4.1 Add `pager: Option<String>` to `Rendered` (default None; existing commands unchanged)
- [x] 4.2 Diff fills it: TTY and no `--no-pager` → resolve via `git var GIT_PAGER` from the VMR root through the runner seam (`cat`/empty → None)
- [x] 4.3 Emit pages: spawn `sh -c` with `LESS=FRX`/`LV=-c` when unset, write stdout to pager stdin, wait, then print stderr; no re-print on pager death; direct-print fallback if `sh` cannot spawn
- [x] 4.4 Tests: scripted fake answers `var GIT_PAGER`; emit tests use real `sh -c` recording scripts asserting verbatim stdin and stderr-after-exit ordering

## 5. Integration and the applyability contract ([ticket 09](tickets/09-integration-apply-round-trip.md))

- [x] 5.1 `tests/diff.rs` via `assert_cmd`: worktree diff, `--staged`, path filters (owned/aggregate/unowned-error), one-failing-repo aggregation, uncoloured unpaged piped output
- [x] 5.2 Apply round-trip: real edits plus a real rename (plus binary/mode-only/quoted-path if practical) → `git vmr diff` → `git apply -p1` onto a pristine VMR copy → trees match

## 6. Pager shell resolution and the argv seam ([ticket 12](tickets/12-pager-shell-resolution.md))

- [ ] 6.1 Change `Rendered.pager` to `Option<Vec<String>>`; emit spawns argv[0] with the remaining args verbatim (no shell/`-c` knowledge in emit)
- [ ] 6.2 Diff composes the argv: `git var GIT_SHELL_PATH` through the runner seam (uniform, no `cfg` fork), literal `"sh"` on failure, then `-c` plus the pager command
- [ ] 6.3 Emit prepends `dirname(argv[0])` to the pager child's PATH; `LESS=FRX`/`LV=-c` defaults stay
- [ ] 6.4 Graduate the direct-print fallback: silent, designed, tested
- [ ] 6.5 Tests: scripted-fake coverage of the shell chain; recording script echoing `$PATH` proves the prepend; unspawnable argv[0] proves silent direct print with exit code unchanged

## 7. Windows VT probe gating auto-colour ([ticket 13](tickets/13-windows-vt-colour.md))

- [ ] 7.1 Attempt `ENABLE_VIRTUAL_TERMINAL_PROCESSING` at startup behind `cfg(windows)`; constant `true` on Unix
- [ ] 7.2 Feed the capability into colour resolution as an injected fact beside the TTY bool: `--color=auto` → `never` when VT is unavailable; explicit `always` passes through; paging unaffected
- [ ] 7.3 Unit tests flip both injected facts to cover the resolution matrix

## 8. Windows CI test job ([ticket 14](tickets/14-windows-ci.md))

- [ ] 8.1 Add a `windows-latest` job to `ci.yml` running the test suite, noting the friendly-runner-PATH caveat (proves honest breakage, not the degradation paths)
