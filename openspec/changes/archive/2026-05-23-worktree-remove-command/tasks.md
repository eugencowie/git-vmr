## 1. CLI Surface

- [x] 1.1 Add a `Remove { force, path }` variant to the nested `WorktreeCommand` enum.
- [x] 1.2 Parse `-f` and `--force` as a counted flag so repeated occurrences are preserved.
- [x] 1.3 Dispatch `git vmr worktree remove [-f|--force] <worktree>` to the worktree command module.
- [x] 1.4 Add parser tests for required worktree path, rejected extra operands, single force, repeated long force, and repeated short force.
- [x] 1.5 Update the unsupported worktree subcommand parser test so `remove` is accepted and another unsupported subcommand remains rejected.

## 2. Git Delegation

- [x] 2.1 Add a Git helper for `git worktree remove <target>`.
- [x] 2.2 Forward the counted force occurrences by passing `-f` once per occurrence to Git before the target path.
- [x] 2.3 Reuse existing first-non-empty-line and repository-suffixed aggregate reporting behavior for worktree remove success and failure output.
- [x] 2.4 Preserve Git failure text for dirty, locked, missing, and otherwise invalid worktree removal attempts.

## 3. VMR Worktree Removal

- [x] 3.1 Resolve the aggregate worktree path relative to the effective working directory.
- [x] 3.2 Discover immediate child Git repositories from the source VMR root and skip non-Git children.
- [x] 3.3 Remove each child worktree at `<aggregate-worktree>/<repo-name>` using best-effort execution with no rollback.
- [x] 3.4 Leave successful child removals in place when other child removals fail.
- [x] 3.5 After all child removals succeed, remove the aggregate `.gitvmr` marker if present.
- [x] 3.6 After all child removals succeed, remove the aggregate directory if it is empty.
- [x] 3.7 Leave the aggregate `.gitvmr` marker and aggregate directory in place when any child removal fails.

## 4. Tests

- [x] 4.1 Test `git vmr worktree remove ../wt` removes `../wt/backend` and `../wt/frontend`.
- [x] 4.2 Test non-Git child directories are skipped during removal.
- [x] 4.3 Test dirty child worktree removal fails without force and reports the Git failure with repository suffixes.
- [x] 4.4 Test `--force` removes dirty child worktrees by forwarding one force flag.
- [x] 4.5 Test `--force --force` removes locked child worktrees by forwarding two force flags.
- [x] 4.6 Test `-ff` is counted as two force flags.
- [x] 4.7 Test best-effort behavior leaves successful child removals removed when another child repository fails.
- [x] 4.8 Test successful aggregate removal removes `.gitvmr` and removes the empty aggregate directory.
- [x] 4.9 Test partial aggregate removal keeps `.gitvmr` in place.
- [x] 4.10 Test nested current directory and global `-C` behavior for source VMR discovery and relative aggregate worktree path resolution.
- [x] 4.11 Test deterministic failure ordering for multiple child repository failures.

## 5. Documentation and Verification

- [x] 5.1 Update `docs/git-vmr/worktree.md` to mark `worktree remove [-f] <worktree>` and force flag support.
- [x] 5.2 Update any top-level command support table entries if needed.
- [x] 5.3 Run `cargo fmt`.
- [x] 5.4 Run focused CLI and worktree tests.
- [x] 5.5 Run the full test suite.
- [x] 5.6 Run `nix develop -c openspec validate worktree-remove-command --strict`.
