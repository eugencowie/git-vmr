## 1. CLI Wiring

- [x] 1.1 Add a `Clone` subcommand with required `<repo>` and optional `<dir>` arguments to the clap command enum.
- [x] 1.2 Add a clone command module and route `Command::Clone` from `Cli::run` using the resolved working directory.
- [x] 1.3 Add parser unit coverage for `git vmr clone <repo>`, `git vmr clone <repo> <dir>`, and `git vmr -C <path> clone <repo> <dir>`.

## 2. Passthrough Execution

- [x] 2.1 Invoke `git -C <working-dir> clone <repo> [<dir>]` without VMR root discovery.
- [x] 2.2 Use inherited stdin, stdout, and stderr so Git clone progress, prompts, and errors remain visible.
- [x] 2.3 Return success only when the Git clone process exits successfully and return a non-zero command failure when Git exits unsuccessfully.
- [x] 2.4 Surface process launch failures with enough context to identify the attempted clone.

## 3. Integration Tests

- [x] 3.1 Test cloning a local repository into Git's inferred destination directory from a non-VMR directory.
- [x] 3.2 Test cloning a local repository into an explicit destination directory.
- [x] 3.3 Test `-C <path>` makes a relative destination directory resolve under the effective working directory.
- [x] 3.4 Test clone does not require or create `.gitvmr/` metadata.
- [x] 3.5 Test a failed Git clone exits non-zero.

## 4. Documentation

- [x] 4.1 Update README command tables to mark `clone` as supported.
- [x] 4.2 Update `docs/git-vmr.md` and add clone command documentation for the supported syntax.

## 5. Verification

- [x] 5.1 Run `cargo fmt`.
- [x] 5.2 Run `cargo test`.
- [x] 5.3 Run `nix develop -c openspec status --change "clone-command"` and confirm the change is apply-ready.
