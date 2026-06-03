## 1. CLI Wiring

- [x] 1.1 Add a `Merge` subcommand with a required `<commit-ish>` argument to the clap command enum.
- [x] 1.2 Add a `src/cli/merge.rs` module and route `Command::Merge` to it from `Cli::run`.

## 2. Merge Execution

- [x] 2.1 Implement immediate child repository discovery that skips `.gitvmr/` and non-Git child directories.
- [x] 2.2 Run `git --no-optional-locks -C <repo> merge <commit-ish>` in every discovered child Git repository in deterministic repository-name order.
- [x] 2.3 Preserve Git's normal per-repository behavior without pre-filtering refs, rolling back successful merges, or aborting conflicted merges.

## 3. Reporting and Errors

- [x] 3.1 Capture per-repository merge successes and print the first non-empty Git stdout line with the repository name appended.
- [x] 3.2 Capture per-repository merge failures using the first non-empty Git stderr line, falling back to stdout when stderr is empty, with the repository name appended.
- [x] 3.3 Aggregate failures after all repositories have been attempted and return a non-zero status if any merge fails.
- [x] 3.4 Succeed with no output when no immediate child Git repositories are found.

## 4. Integration Tests

- [x] 4.1 Test clean merges across multiple child repositories.
- [x] 4.2 Test non-Git child directories are skipped and empty VMRs succeed with no output.
- [x] 4.3 Test missing refs fail only the affected repository while other repositories are still attempted.
- [x] 4.4 Test a conflicted merge leaves the affected repository in conflicted merge state and does not stop clean repositories.
- [x] 4.5 Test successful merges are not rolled back after a later repository fails.
- [x] 4.6 Test repository-suffixed success and failure output, including deterministic failure ordering.
- [x] 4.7 Test nested working directory and global `-C <path>` behavior.

## 5. Verification

- [x] 5.1 Run `cargo fmt`.
- [x] 5.2 Run `cargo test`.
- [x] 5.3 Run `nix develop -c openspec status --change "merge-command"` and confirm the change is apply-ready.
