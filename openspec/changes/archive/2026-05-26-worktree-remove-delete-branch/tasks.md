## 1. CLI and Git Plumbing

- [x] 1.1 Add a `delete` boolean flag to `WorktreeCommand::Remove` with `-d` and `--delete` parsing.
- [x] 1.2 Update command dispatch so `worktree::remove` receives the delete flag.
- [x] 1.3 Add or reuse Git helpers needed to read the target child worktree branch state before removal.
- [x] 1.4 Reuse safe `git branch -d` delegation for requested branch deletion after successful child worktree removal.

## 2. Remove Orchestration

- [x] 2.1 Refactor `src/commands/worktree.rs` remove flow to retain per-repository removal outcomes instead of bailing immediately after `print_results`.
- [x] 2.2 Record each target child worktree's branch or detached state before removal when `--delete` is requested.
- [x] 2.3 Run branch deletion only for repositories whose child worktree removal succeeded and whose target worktree was branch-backed.
- [x] 2.4 Preserve aggregate marker and directory cleanup when all child worktree removals succeed, regardless of branch deletion success.
- [x] 2.5 Combine worktree removal and branch deletion reporting while keeping deterministic repository-name ordering.

## 3. Tests

- [x] 3.1 Add CLI parser tests for `worktree remove -d ../wt` and `worktree remove --delete ../wt`.
- [x] 3.2 Add integration tests for successful branch deletion after aggregate worktree removal.
- [x] 3.3 Add integration tests proving actual child worktree branch names are used instead of aggregate path basenames.
- [x] 3.4 Add integration tests for detached child worktrees skipping branch deletion.
- [x] 3.5 Add integration tests for partial removal behavior: delete branches for successful repositories and skip failed repositories.
- [x] 3.6 Add integration tests for branch deletion failure after successful worktree removal, including aggregate marker cleanup.
- [x] 3.7 Add integration tests proving `--force --delete` does not force-delete unmerged branches.
- [x] 3.8 Add deterministic failure-order coverage for branch deletion failures.

## 4. Documentation and Verification

- [x] 4.1 Update `docs/git-vmr/worktree.md` to document `-d | --delete` for `worktree remove`.
- [x] 4.2 Run focused worktree parser and integration tests.
- [x] 4.3 Run the full test suite.
- [x] 4.4 Run `openspec status --change worktree-remove-delete-branch` and confirm the change is apply-ready.
