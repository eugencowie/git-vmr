## 1. CLI Wiring

- [x] 1.1 Add a `Rebase` subcommand with a required `<upstream>` argument to the clap command enum.
- [x] 1.2 Add a `src/cli/rebase.rs` module and route `Command::Rebase` to it from `Cli::run`.

## 2. Rebase Execution

- [x] 2.1 Implement immediate child repository discovery that skips `.gitvmr/` and non-Git child directories.
- [x] 2.2 Run `git --no-optional-locks -C <repo> rebase <upstream>` for every discovered child Git repository.
- [x] 2.3 Execute child repository rebase attempts in parallel and capture each repository's stdout, stderr, and exit status.
- [x] 2.4 Preserve Git's normal per-repository behavior without pre-filtering upstreams, rolling back successful rebases, or aborting conflicted rebases.
- [x] 2.5 Succeed with no output when all attempted rebases succeed or when no immediate child Git repositories are found.

## 3. Reporting and Errors

- [x] 3.1 Capture per-repository rebase failures using the first non-empty Git stderr line, falling back to stdout when stderr is empty, with the repository name appended.
- [x] 3.2 Sort failure reports deterministically by repository name after parallel attempts complete.
- [x] 3.3 Aggregate failures after all repositories have been attempted and return a non-zero status if any rebase fails.
- [x] 3.4 Ensure successful rebases do not produce aggregate stdout output.

## 4. Integration Tests

- [x] 4.1 Test clean rebases across multiple child repositories succeed quietly.
- [x] 4.2 Test non-Git child directories are skipped and empty VMRs succeed with no output.
- [x] 4.3 Test missing upstreams fail only affected repositories while other repositories are still attempted.
- [x] 4.4 Test a conflicted rebase leaves the affected repository in an in-progress rebase state and does not stop clean repositories.
- [x] 4.5 Test successful rebases are not rolled back after another repository fails.
- [x] 4.6 Test repository-suffixed failure output, including deterministic ordering after parallel execution.
- [x] 4.7 Test nested working directory and global `-C <path>` behavior.

## 5. Verification

- [x] 5.1 Run `cargo fmt`.
- [x] 5.2 Run `cargo test`.
- [x] 5.3 Run `nix develop -c openspec status --change "rebase-command"` and confirm the change is apply-ready.
