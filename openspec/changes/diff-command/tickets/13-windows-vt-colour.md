# Windows VT probe gating auto-colour

Type: implementation
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
