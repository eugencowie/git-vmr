# Pager support at the emit choke point

Type: implementation
Status: resolved
Blocked by: 05

## Task

Implement paging per [spec.md](../spec.md) "Colour and paging" / "The
pager seam".

- `Rendered` (`src/render.rs`) gains `pager: Option<String>`, defaulting
  to `None`; every existing command is untouched in behaviour.
- Diff fills it: when stdout is a TTY and `--no-pager` is absent, resolve
  via one `git var GIT_PAGER` through the Git runner seam from the VMR
  root; `cat` or empty means no paging.
- `emit` pages when the field is `Some`: spawn via `sh -c`, export
  `LESS=FRX` and `LV=-c` when unset, write stdout into the pager's stdin,
  `wait()`, then print stderr and propagate the result. No re-print on
  nonzero pager exit; fall back to direct printing only if `sh` cannot
  spawn (untested defensive code).
- Emit stays git-free and TTY-free.
- Tests: pager resolution via the scripted fake answering
  `var GIT_PAGER`; emit tests spawn real `sh -c` recording scripts
  (e.g. `cat > <tempfile>`) asserting verbatim stdin delivery and
  stderr-after-pager-exit ordering. No pager trait.

## Comments

Implemented (2026-07-18): `Rendered` (`src/render.rs`) gained
`pager: Option<String>` (default `None`; every other command untouched).
The diff command fills it when stdout is a TTY and `--no-pager` is
absent, via one `git var GIT_PAGER` from the VMR root through the runner
seam (`resolve_pager` in `src/commands/diff.rs`); `cat`, an empty
resolution, or a failed lookup mean no paging, and the field also rides
the `Failed` path so aggregated failures print after the pager exits.
Emit pages at the choke point (`page` in `src/render.rs`): `sh -c`,
`LESS=FRX`/`LV=-c` exported only when unset, stdout written to the
pager's stdin, `wait()`, then stderr — no re-print on pager death, and a
direct-print fallback only if `sh` cannot spawn. Tests: scripted-fake
resolution coverage (TTY/`--no-pager`/piped paths prove the lookup only
runs when paging is active; `cat`/empty/failed map to `None`) plus emit
tests spawning real `sh -c` recording scripts asserting verbatim stdin
delivery, wait-before-return ordering, env defaults, and swallowed pager
death.
