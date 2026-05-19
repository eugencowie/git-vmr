## 1. CLI Parsing

- [x] 1.1 Add `-c` / `--create` parsing to the `switch` command while keeping the single branch name positional required.
- [x] 1.2 Add parser tests for `switch --create feature/auth`, `switch -c feature/auth`, missing branch name, and rejected start-point arguments.
- [x] 1.3 Update command dispatch so plain switch and create switch route to distinct execution paths.

## 2. Git Execution

- [x] 2.1 Add a Git helper that invokes `git switch --create <new-branch>` for one child repository.
- [x] 2.2 Preserve Git-delegated validation and failure extraction for create-and-switch attempts.
- [x] 2.3 Keep plain `git vmr switch <branch-name>` behavior unchanged.

## 3. Aggregate Reporting

- [x] 3.1 Add create-mode orchestration that runs child repository attempts in parallel and preserves deterministic result ordering.
- [x] 3.2 Deduplicate identical successful create summary lines and print each unique line once without repository suffixes.
- [x] 3.3 Report failed create-and-switch attempts with repository suffixes and return non-zero when any repository fails.

## 4. Tests

- [x] 4.1 Add integration tests for successful `switch --create` across multiple child repositories with one deduplicated success line.
- [x] 4.2 Add integration tests for the short `switch -c` form.
- [x] 4.3 Add integration tests for skipped non-Git children and empty VMR behavior during create.
- [x] 4.4 Add integration tests for partial failure, existing branch failures, deterministic failure ordering, and no rollback after failures.

## 5. Documentation and Validation

- [x] 5.1 Update switch command documentation and support tables for `-c` / `--create`.
- [x] 5.2 Run formatting and the relevant Rust test suite.
- [x] 5.3 Run `openspec validate switch-create-argument --strict`.
