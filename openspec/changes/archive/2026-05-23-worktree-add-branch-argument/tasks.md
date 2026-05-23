## 1. CLI Parsing

- [x] 1.1 Add a short-only `-b <new-branch>` option to the nested `worktree add` command.
- [x] 1.2 Thread the parsed branch option through `Command::Worktree` dispatch into the worktree add command handler.
- [x] 1.3 Add CLI parser tests for `worktree add -b feature/auth ../wt`, `worktree add -b feature/auth ../wt main`, missing `-b` value, and rejected `--branch`.

## 2. Worktree Add Behavior

- [x] 2.1 Update branch selection so explicit `-b <new-branch>` overrides aggregate target basename inference.
- [x] 2.2 Preserve inferred branch behavior when both `-b` and `<commit-ish>` are omitted.
- [x] 2.3 Preserve direct explicit `<commit-ish>` behavior when `-b` is omitted.
- [x] 2.4 Ensure `git worktree add -b <new-branch> <target>/<repo-name> [<commit-ish>]` is delegated to Git without preflight validation.

## 3. Integration Tests

- [x] 3.1 Add an integration test that `git vmr worktree add -b feature/auth ../wt` creates every child worktree on `feature/auth` and does not create branch `wt`.
- [x] 3.2 Add an integration test that `git vmr worktree add -b feature/auth ../wt main` uses `main` as the branch start point.
- [x] 3.3 Add an integration test that explicit branch creation failures are reported with repository suffixes and do not prevent attempts in other child repositories.

## 4. Documentation And Validation

- [x] 4.1 Update `docs/git-vmr/worktree.md` to mark `-b <new-branch>` support and keep `-B` unsupported.
- [x] 4.2 Run `nix develop -c cargo test`.
- [x] 4.3 Run `nix develop -c openspec validate worktree-add-branch-argument --strict`.
