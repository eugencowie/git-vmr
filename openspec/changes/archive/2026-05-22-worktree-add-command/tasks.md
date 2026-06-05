## 1. CLI Surface

- [x] 1.1 Add a nested `Worktree` command enum with an `Add { path, commit_ish }` action to the clap command model.
- [x] 1.2 Dispatch `git vmr worktree add <path> [<commit-ish>]` to a new worktree command module.
- [x] 1.3 Add parser tests for required target path, optional single commit-ish, rejected extra operands, and rejected unsupported worktree subcommands.

## 2. Git Delegation

- [x] 2.1 Add Git helper support for `git worktree add -b <branch> <target>` when commit-ish is omitted.
- [x] 2.2 Add Git helper support for `git worktree add <target> <commit-ish>` when commit-ish is provided.
- [x] 2.3 Reuse existing first-non-empty-line and repository-suffixed aggregate reporting behavior for worktree add success and failure output.

## 3. VMR Worktree Creation

- [x] 3.1 Resolve the aggregate target path relative to the effective working directory.
- [x] 3.2 Discover immediate child Git repositories from the source VMR root and skip non-Git children.
- [x] 3.3 Derive the omitted commit-ish branch name from the aggregate target path basename and use the same branch name for every child repository.
- [x] 3.4 Create each child worktree at `<aggregate-target>/<repo-name>` using best-effort execution with no rollback.
- [x] 3.5 Create a lightweight `.gitvmr` marker at the aggregate target so linked worktrees participate in existing VMR root discovery.

## 4. Tests

- [x] 4.1 Test `git vmr worktree add ../wt` creates `../wt/backend` and `../wt/frontend` on branch `wt`.
- [x] 4.2 Test non-Git child directories are skipped and do not create target children.
- [x] 4.3 Test `git vmr worktree add ../wt main` delegates explicit commit-ish behavior to Git and reports checked-out branch failures with repository suffixes.
- [x] 4.4 Test `git vmr worktree add ../wt new` reports invalid reference failures with grouped repository suffixes and does not create branch `wt`.
- [x] 4.5 Test best-effort behavior leaves successful child worktrees in place when another child repository fails.
- [x] 4.6 Test the linked aggregate target contains `.gitvmr` and commands can discover the linked VMR root from inside a child worktree.
- [x] 4.7 Test nested current directory and global `-C` behavior for source VMR discovery and relative aggregate target path resolution.
- [x] 4.8 Test deterministic failure ordering for multiple child repository failures.

## 5. Documentation and Verification

- [x] 5.1 Add `worktree` command support documentation consistent with existing `docs/git-vmr/*.md` files.
- [x] 5.2 Run `cargo fmt`.
- [x] 5.3 Run the focused worktree tests.
- [x] 5.4 Run the full test suite.
