# CLI surface and the injected TTY fact

Type: implementation
Status: open

## Task

Add the `diff` subcommand skeleton and the TTY fact, per
[spec.md](../spec.md) "Command surface" and "Colour and paging".

- Clap definition: `--staged` (with `--cached` alias), `--color[=<when>]`
  (`auto`/`always`/`never`, bare flag = `always`, default `auto`),
  `--no-pager`, trailing `<path>...`. Follow the existing subcommand
  patterns in `src/cli.rs` / `src/commands.rs`; wire event metadata like
  the other commands (names only, no values).
- Add the injected TTY bool to `CliContext` (`src/cli/context.rs`):
  production sets it from `stdout().is_terminal()`; `for_tests` defaults
  it to `false`.
- `src/commands/diff.rs` with a `run(workspace, context)` returning
  `Rendered` — stub the diff body; resolve `--color=auto` against the TTY
  fact here so later tickets consume a resolved `always`/`never`.
- Unit tests for the colour resolution logic (TTY bool flipped in tests).

Done when the command parses, routes nothing yet, and colour/pager
gating decisions are unit-tested.
