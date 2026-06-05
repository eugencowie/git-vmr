## 1. CLI Surface

- [x] 1.1 Add a `reset` CLI module declaration and command enum variant.
- [x] 1.2 Parse `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` with mutually exclusive mode flags.
- [x] 1.3 Dispatch the parsed reset mode and optional commit from `Cli::run`.
- [x] 1.4 Add CLI parser tests for no arguments, each supported mode, mode plus commit, `-C`, conflicting modes, and unsupported extra arguments.

## 2. Reset Execution

- [x] 2.1 Add a reset mode representation that can be shared between CLI parsing and Git invocation.
- [x] 2.2 Add `src/cli/reset.rs` to discover the VMR root, collect child repositories, run reset attempts in parallel, and use existing aggregate result printing.
- [x] 2.3 Add `src/git/reset.rs` to build and run `git reset [mode] [<commit>]` for one child repository.
- [x] 2.4 Export the reset helper from `src/git.rs` and wire the CLI module into `src/cli.rs`.

## 3. Behavior Tests

- [x] 3.1 Add integration tests showing reset applies across multiple child repositories and skips non-Git children.
- [x] 3.2 Add integration tests for optional commit handling, no-commit resets, and each supported mode.
- [x] 3.3 Add integration tests showing missing refs or rejected reset states fail only affected repositories.
- [x] 3.4 Add integration tests proving successful resets are not rolled back after another repository fails.
- [x] 3.5 Add integration tests for deterministic repository-suffixed success and failure reporting.
- [x] 3.6 Add integration tests for running from inside a child repository and with global `-C`.

## 4. Documentation

- [x] 4.1 Generate or add reset command documentation at `docs/git-vmr/reset.md`.
- [x] 4.2 Update `docs/git-vmr.md` to link `reset` and mark it supported.
- [x] 4.3 Update `README.md` if its command support list or examples mention supported porcelain commands.

## 5. Validation

- [x] 5.1 Run formatting checks for the Rust changes.
- [x] 5.2 Run the reset integration tests.
- [x] 5.3 Run the full test suite or the repository's standard CI-equivalent test command.
- [x] 5.4 Run `openspec validate reset-command --strict`.
