## 1. CLI Wiring

- [x] 1.1 Add a `Branch` subcommand to the clap command enum with Git-compatible help text.
- [x] 1.2 Add a new `src/cli/branch.rs` module and dispatch `Command::Branch` from `Cli::run`.

## 2. Branch Collection

- [x] 2.1 Discover the VMR root from the effective working directory using the existing VMR root discovery function.
- [x] 2.2 Scan immediate child directories, skip non-Git children, and keep repository ordering deterministic.
- [x] 2.3 Collect local branch refs from each child Git repository with Git CLI commands scoped to `refs/heads`.
- [x] 2.4 Detect each repository's active branch with `git symbolic-ref --quiet --short HEAD`.
- [x] 2.5 Detect detached HEAD repositories and collect short commit hashes for detached output lines.
- [x] 2.6 Return an error when a child that appears to be a Git repository cannot provide branch information.

## 3. Rendering

- [x] 3.1 Group collected branch refs by branch name across discovered child repositories.
- [x] 3.2 Render `*` for branch lines checked out in at least one repository and a leading space for inactive branch lines.
- [x] 3.3 Omit repository names for branches that exist in every discovered child Git repository.
- [x] 3.4 Include sorted repository names for branches that exist in only a subset of discovered child Git repositories.
- [x] 3.5 Render detached HEAD repositories as `* (HEAD detached at <short-hash>) (<repo-name>)` lines.
- [x] 3.6 Ensure no child repositories or no local branch refs produces successful empty output.

## 4. Tests

- [x] 4.1 Add rendering unit tests for shared branches, partial branches, active markers, inactive markers, and detached HEAD lines.
- [x] 4.2 Add integration tests for `git vmr branch` across multiple child repositories with local-only branch inventory.
- [x] 4.3 Add integration tests for non-Git child skipping and no-child-repository empty output.
- [x] 4.4 Add integration tests for nested working directory discovery and global `-C <path>` behavior.
- [x] 4.5 Add an integration test for corrupted child Git repository failure.

## 5. Verification

- [x] 5.1 Run `nix develop -c cargo fmt --check`.
- [x] 5.2 Run `nix develop -c cargo test`.
- [x] 5.3 Run `nix develop -c openspec validate branch-command --strict`.
