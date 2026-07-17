# Pager support at the emit choke point

Type: implementation
Status: open
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
