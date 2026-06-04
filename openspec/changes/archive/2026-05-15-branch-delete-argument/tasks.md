## 1. CLI Parsing

- [x] 1.1 Replace the branch subcommand's bare optional branch name dispatch shape with an explicit list/create/delete/force-delete action.
- [x] 1.2 Add `-d` and `-D` flags for the branch subcommand with mutual exclusion.
- [x] 1.3 Validate that `-d` and `-D` each require exactly one branch name argument.
- [x] 1.4 Add CLI parser tests for safe deletion, force deletion, missing branch name, and conflicting delete flags.

## 2. Git Execution

- [x] 2.1 Add a Git branch deletion helper that runs `git branch -d <branch-name>` or `git branch -D <branch-name>` in a child repository.
- [x] 2.2 Preserve Git's first successful deletion output line and suffix it with the repository name.
- [x] 2.3 Preserve Git's first failure output line and suffix it with the repository name.

## 3. Aggregate Branch Command

- [x] 3.1 Dispatch safe deletion and force deletion from `git vmr branch` to the new Git helper.
- [x] 3.2 Attempt deletion independently in every discovered child Git repository.
- [x] 3.3 Skip non-Git child directories during deletion using existing VMR repository discovery.
- [x] 3.4 Return non-zero after all deletion attempts finish when any repository fails.
- [x] 3.5 Keep branch listing and branch creation behavior unchanged.

## 4. Tests

- [x] 4.1 Add integration tests for successful safe deletion across multiple child repositories.
- [x] 4.2 Add integration tests for force deletion of a branch that safe deletion would reject.
- [x] 4.3 Add integration tests showing non-Git child directories are skipped during deletion.
- [x] 4.4 Add integration tests showing partial deletion failures do not stop successful deletion in other repositories.
- [x] 4.5 Add integration tests for concise missing-branch or checked-out-branch failure reports.
- [x] 4.6 Add integration tests proving multiple deletion failures are reported in deterministic repository-name order.

## 5. Validation

- [x] 5.1 Run `nix develop -c cargo fmt -- --check`.
- [x] 5.2 Run `nix develop -c cargo test`.
- [x] 5.3 Run `nix develop -c openspec validate branch-delete-argument --strict`.
