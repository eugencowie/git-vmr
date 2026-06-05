## 1. CLI Parsing

- [x] 1.1 Add short-only `-D` parsing to `git vmr worktree remove`.
- [x] 1.2 Ensure `-D` conflicts with `-d | --delete`.
- [x] 1.3 Ensure no long force-delete option is accepted.
- [x] 1.4 Add parser tests for `-D`, `--force -D`, `-d -D` rejection, and long force-delete rejection.

## 2. Command Behavior

- [x] 2.1 Represent worktree remove branch deletion as none, safe delete, or force delete.
- [x] 2.2 Pass forced branch deletion through to the existing Git branch deletion helper after successful child worktree removal.
- [x] 2.3 Preserve `-f | --force` as the repeated force count for `git worktree remove` only.
- [x] 2.4 Preserve existing detached worktree, failed removal, partial failure, result reporting, and aggregate cleanup behavior.

## 3. Integration Tests

- [x] 3.1 Test `git vmr worktree remove -D ../wt` force-deletes an unmerged checked-out branch after removing the child worktree.
- [x] 3.2 Test `git vmr worktree remove --force -D ../wt` combines forced worktree removal with forced branch deletion.
- [x] 3.3 Test `git vmr worktree remove --force --delete ../wt` still uses safe branch deletion and fails for an unmerged branch.
- [x] 3.4 Test `-D` uses the actual child worktree branch rather than the aggregate path basename.
- [x] 3.5 Test `-D` skips branch deletion for detached child worktrees and for repositories whose worktree removal failed.

## 4. Documentation and Validation

- [x] 4.1 Update `docs/git-vmr/worktree.md` to document short-only `-D` support for remove.
- [x] 4.2 Run focused CLI and worktree tests.
- [x] 4.3 Run `nix develop -c openspec validate worktree-remove-force-delete-branch --strict`.
