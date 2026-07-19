# Windows VT probe gating auto-colour

Type: implementation
Status: resolved
Blocked by: 11

## Task

Implement the colour decision from
[Decide Windows behaviour for paging, colour, and TTY](11-windows-behavior.md)
per [spec.md](../spec.md) "Windows":

- On Windows startup, attempt to enable
  `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on the console output handle
  (small guarded `SetConsoleMode` call behind `cfg(windows)`, or the
  `enable-ansi-support` crate).
- On failure (legacy conhost), `--color=auto` resolves to `never`
  instead of emitting unrendered escapes. Explicit `--color=always`
  passes through untouched.
- VT capability gates **auto-colour only** — paging still gates on the
  TTY fact alone. On Unix the probe is a constant `true`; keep the
  capability an injected fact beside the TTY bool so the resolution
  logic stays unit-testable on Linux (both values flipped in tests).

## Answer

Implemented (2026-07-18): `CliContext` (`src/cli/context.rs`) gained
`stdout_supports_color`, filled at startup by a probe that is constant
`true` off Windows; the `cfg(windows)` arm is a small guarded
`GetConsoleMode`/`SetConsoleMode` pair (via `windows-sys`, a
target-gated dependency) that ORs in
`ENABLE_VIRTUAL_TERMINAL_PROCESSING` on the stdout handle. A refused
`SetConsoleMode` (legacy conhost) reads false; a failed `GetConsoleMode`
(pipes, MSYS PTYs — not a console at all) reads **true**, leaving the
TTY fact in charge so Git Bash keeps auto-colour. The
`enable-ansi-support`/`anstyle-query` crates were rejected because they
report non-console stdout as incapable, which would strip Git Bash.
`resolve_color` in `src/commands/diff.rs` takes both facts:
`--color=auto` needs TTY **and** VT; explicit choices pass through;
`paging_active` still sees the TTY bool alone. Unit tests flip both
facts (conhost TTY-without-VT resolves `never`); `for_tests` contexts
default the capability to true so the Unix suite is unchanged.
