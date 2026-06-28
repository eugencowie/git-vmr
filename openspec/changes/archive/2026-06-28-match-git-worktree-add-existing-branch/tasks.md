## 1. Git Delegation

- [x] 1.1 Add a focused Git helper for checking whether `refs/heads/<branch>` exists in a child repository.
- [x] 1.2 Update worktree add delegation so inferred existing local branches use `git worktree add <target> <branch>`.
- [x] 1.3 Keep inferred absent branches on `git worktree add -b <branch> <target>`.
- [x] 1.4 Preserve explicit `-b <new-branch>` and explicit `<commit-ish>` delegation paths unchanged.

## 2. Tests

- [x] 2.1 Update the existing omitted-commit-ish test to cover creating branch `wt` when absent.
- [x] 2.2 Add a test that `worktree add ../wt` checks out existing local branch `wt` instead of failing.
- [x] 2.3 Add a test for mixed child state where one repository checks out existing `wt` and another creates it.
- [x] 2.4 Add a test that an existing inferred branch checked out elsewhere fails with Git's checked-out branch error.
- [x] 2.5 Keep or update explicit `-b <branch>` branch-exists coverage to prove create-only behavior remains.

## 3. Documentation and Verification

- [x] 3.1 Update `docs/git-vmr/worktree.md` to describe omitted-commit-ish existing-branch checkout behavior.
- [x] 3.2 Run `mise exec -- cargo +nightly fmt --check`.
- [x] 3.3 Run focused worktree tests.
- [x] 3.4 Run the full test suite.
