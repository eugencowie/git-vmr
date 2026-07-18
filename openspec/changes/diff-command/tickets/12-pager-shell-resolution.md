# Pager shell resolution and the argv seam

Type: implementation
Blocked by: 08, 11

## Task

Implement the pager decisions from
[Decide Windows behaviour for paging, colour, and TTY](11-windows-behavior.md)
per [spec.md](../spec.md) "The pager seam":

- `Rendered.pager` changes shape: `Option<String>` →
  `Option<Vec<String>>` (argv). Emit spawns argv[0] with the remaining
  args verbatim — it no longer knows about shells or `-c`.
- The diff command composes the argv: resolve the shell via one
  `git var GIT_SHELL_PATH` through the runner seam (uniform on all
  platforms, no `cfg` fork); on failure (git < 2.45) fall back to the
  literal `"sh"`. Result: `[<shell>, "-c", <pager command>]`.
- Emit prepends `dirname(argv[0])` to the pager child's `PATH` (so Git
  for Windows' `less` resolves beside its `sh`; harmless on Unix).
  `LESS=FRX`/`LV=-c` defaults stay in emit.
- The direct-print fallback on spawn failure graduates from untested
  defensive code to designed, **silent**, tested behaviour.
- Tests: scripted-fake coverage of the `GIT_SHELL_PATH` → `"sh"` chain;
  an emit recording script echoing `$PATH` proving the prepend; an
  unspawnable argv[0] proving silent direct print with the exit code
  unchanged. Existing `sh -c` recording-script tests carry over on the
  argv shape.
