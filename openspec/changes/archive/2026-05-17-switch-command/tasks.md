## 1. CLI Surface

- [x] 1.1 Add a `switch` module declaration to `src/cli.rs`.
- [x] 1.2 Add a `Switch { branch_name: String }` subcommand with required `<branch-name>` parsing.
- [x] 1.3 Dispatch `git vmr switch <branch-name>` to the switch command orchestration function.
- [x] 1.4 Add CLI parser tests for the switch subcommand and required branch name.

## 2. Git Integration

- [x] 2.1 Add a `src/git/switch.rs` helper that runs `git switch <branch-name>` in a child repository.
- [x] 2.2 Export the switch helper from `src/git.rs`.
- [x] 2.3 Capture success output from Git stdout or stderr and append the repository suffix.
- [x] 2.4 Capture failure output from Git stderr with stdout fallback and append the repository suffix.

## 3. Command Orchestration

- [x] 3.1 Add `src/cli/switch.rs` to find the VMR root and discover immediate child Git repositories.
- [x] 3.2 Run per-repository switch attempts with `rayon::par_iter`.
- [x] 3.3 Reuse the aggregate result path so successes are printed and all errors are reported after attempts complete.
- [x] 3.4 Preserve deterministic reporting by relying on sorted repository discovery and collected result order.
- [x] 3.5 Ensure non-Git child directories are skipped and empty VMRs succeed quietly.

## 4. Tests

- [x] 4.1 Add integration tests for switching a shared branch across multiple child repositories.
- [x] 4.2 Add integration tests for skipping non-Git child directories and succeeding quietly in empty VMRs.
- [x] 4.3 Add integration tests proving missing branch failures do not stop successful repositories.
- [x] 4.4 Add integration tests proving successful switches are not rolled back after another repository fails.
- [x] 4.5 Add integration tests for deterministic repository-suffixed failure reporting.
- [x] 4.6 Add integration tests for dirty-worktree protection delegated to Git.
- [x] 4.7 Add integration tests for nested working directory discovery and the global `-C` option.

## 5. Documentation and Validation

- [x] 5.1 Update README and command documentation support tables to mark `switch` as supported.
- [x] 5.2 Add or update generated command documentation for `git-vmr switch`.
- [x] 5.3 Run `nix develop -c cargo fmt`.
- [x] 5.4 Run `nix develop -c cargo test`.
- [x] 5.5 Run `nix develop -c openspec validate switch-command --strict`.
