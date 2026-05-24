## 1. CLI Wiring

- [x] 1.1 Add a `Foreach` subcommand to the clap command enum with a `--quiet` flag and one-or-more trailing command arguments.
- [x] 1.2 Add a new `src/commands/foreach.rs` module and dispatch `Command::Foreach` from `Command::run`.
- [x] 1.3 Add CLI parsing tests for missing command rejection, quiet mode parsing, multi-word command capture, and child command option capture.

## 2. Child Command Execution

- [x] 2.1 Discover the VMR root from the effective working directory and scan immediate child Git repositories with existing VMR helpers.
- [x] 2.2 Implement parallel child process execution with each process working directory set to the child repository root.
- [x] 2.3 Evaluate the captured command through the shell as one command string.
- [x] 2.4 Set child stdin to empty/null while capturing stdout, stderr, and exit status.
- [x] 2.5 Set `name`, `sm_path`, `displaypath`, and `toplevel` environment variables for each child command without synthesizing `sha1`.

## 3. Output and Failure Reporting

- [x] 3.1 Buffer child stdout and stderr until all child commands complete.
- [x] 3.2 Render repository entry headers to stdout in deterministic repository order unless `--quiet` is set.
- [x] 3.3 Replay buffered child stdout to stdout and buffered child stderr to stderr in deterministic repository order.
- [x] 3.4 Report failed repositories and exit statuses on stderr in deterministic repository order.
- [x] 3.5 Exit successfully only when every child command exits successfully.

## 4. Documentation

- [x] 4.1 Add `docs/git-vmr/foreach.md` documenting command syntax, shell evaluation, parallel buffered output, environment variables, quiet mode, stdin behavior, and failure semantics.
- [x] 4.2 Update `README.md` and `docs/git-vmr.md` to list `foreach` as a supported command.

## 5. Tests

- [x] 5.1 Add integration tests for running commands across multiple child repositories and skipping non-Git children.
- [x] 5.2 Add integration tests for empty VMR success and missing VMR failure.
- [x] 5.3 Add integration tests for deterministic output ordering after parallel command completion.
- [x] 5.4 Add integration tests for stdout replay, stderr replay, default headers, and `--quiet`.
- [x] 5.5 Add integration tests for child command failure reporting across one and multiple repositories.
- [x] 5.6 Add integration tests for `name`, `sm_path`, `displaypath`, `toplevel`, and absent `sha1` environment behavior.
- [x] 5.7 Add integration tests for closed stdin behavior.
- [x] 5.8 Add integration tests for nested working directory discovery and global `-C <path>` behavior.

## 6. Verification

- [x] 6.1 Run `nix develop -c cargo fmt --check`.
- [x] 6.2 Run `nix develop -c cargo test`.
- [x] 6.3 Run `nix develop -c openspec validate foreach-command --strict`.
