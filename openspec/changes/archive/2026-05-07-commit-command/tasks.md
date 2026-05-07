## 1. CLI Surface

- [x] 1.1 Add a `Commit` subcommand to the clap command enum with `-m` and `--message` support.
- [x] 1.2 Add `src/cli/commit.rs` and dispatch `Command::Commit` from `Cli::run`.
- [x] 1.3 Add parser coverage for accepted and rejected commit message forms.

## 2. Repository Discovery and Eligibility

- [x] 2.1 Reuse existing VMR root discovery from the effective working directory.
- [x] 2.2 Scan immediate child directories in deterministic repository-name order and skip non-Git directories.
- [x] 2.3 Detect whether each child Git repository has staged changes.
- [x] 2.4 Fail with a concise `nothing to commit` error when no child Git repository has staged changes.

## 3. Commit Execution and Reporting

- [x] 3.1 Run `git commit -m <message>` in every eligible child Git repository.
- [x] 3.2 Attempt all eligible repositories even when one or more commits fail.
- [x] 3.3 Print the first non-empty successful Git commit output line to stdout with the repository name suffix.
- [x] 3.4 Report the first non-empty failed Git commit error line to stderr with the repository name suffix.
- [x] 3.5 Return a non-zero exit status when any attempted repository commit fails.

## 4. Integration Tests

- [x] 4.1 Test committing staged changes in multiple child repositories with repository-suffixed summary output.
- [x] 4.2 Test skipping non-Git child directories and clean child Git repositories.
- [x] 4.3 Test `-m` and `--message` message handling and rejection when no message is provided.
- [x] 4.4 Test initial commit output preserves Git's root-commit summary line with a repository suffix.
- [x] 4.5 Test partial failures still attempt later eligible repositories and report failures deterministically.
- [x] 4.6 Test `nothing to commit` when no child Git repository has staged changes.
- [x] 4.7 Test existing `-C` and nested working directory behavior for commit.

## 5. Validation

- [x] 5.1 Run the focused commit integration tests.
- [x] 5.2 Run the full test suite.
- [x] 5.3 Run `openspec status --change commit-command` and confirm the change is apply-ready.
