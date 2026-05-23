## 1. CLI Surface

- [x] 1.1 Add a `Move` variant to the nested `WorktreeCommand` enum.
- [x] 1.2 Add `-f` and `--force` parsing for `worktree move` using counted force occurrences.
- [x] 1.3 Require exactly `<worktree>` and `<new-path>` operands for `worktree move`.
- [x] 1.4 Route the parsed `Move` command to the worktree command implementation.

## 2. Git Delegation

- [x] 2.1 Add a Git helper for `git worktree move` that accepts repository name, repository path, source child worktree path, destination child worktree path, and force count.
- [x] 2.2 Forward one `-f` argument per force occurrence before the source and destination paths.
- [x] 2.3 Return repository-suffixed success and failure messages using the same output conventions as existing worktree helpers.

## 3. Aggregate Move Implementation

- [x] 3.1 Discover the source VMR from the effective working directory and scan immediate child Git repositories.
- [x] 3.2 Resolve source and destination aggregate paths relative to the effective working directory.
- [x] 3.3 Create the destination aggregate directory before invoking child Git worktree moves.
- [x] 3.4 Create the destination aggregate `.gitvmr` marker before invoking child Git worktree moves.
- [x] 3.5 Move each child worktree from `<worktree>/<repo-name>` to `<new-path>/<repo-name>` using best-effort parallel execution with no rollback.
- [x] 3.6 Skip non-Git child directories during fan-out.
- [x] 3.7 After complete child-move success, remove the source `.gitvmr` marker if present.
- [x] 3.8 After complete child-move success, remove the source aggregate directory if it is empty.
- [x] 3.9 Leave source and destination markers in place when any child move fails, including total child-move failure.

## 4. Tests

- [x] 4.1 Add CLI parser tests for `worktree move <worktree> <new-path>`.
- [x] 4.2 Add CLI parser tests for single and repeated `-f`/`--force` on `worktree move`.
- [x] 4.3 Add CLI parser tests for missing operands and extra operands.
- [x] 4.4 Test `git vmr worktree move ../wt ../moved` moves child worktrees for multiple repositories.
- [x] 4.5 Test non-Git child directories are skipped during move.
- [x] 4.6 Test dirty child worktree move failure is delegated to Git and reported with the repository suffix.
- [x] 4.7 Test `--force` forwards to Git and moves a dirty child worktree when Git allows it.
- [x] 4.8 Test best-effort behavior keeps successful child moves when another child fails.
- [x] 4.9 Test successful aggregate move removes the source marker and empty source aggregate directory.
- [x] 4.10 Test partial aggregate move keeps both source and destination markers.
- [x] 4.11 Test total aggregate move failure leaves the destination marker.
- [x] 4.12 Test nested current directory and global `-C` behavior for source VMR discovery and relative source/destination aggregate path resolution.
- [x] 4.13 Test worktree move failures are reported in deterministic repository name order.

## 5. Documentation and Validation

- [x] 5.1 Update `docs/git-vmr/worktree.md` to mark `worktree move [-f] <worktree> <new-path>` and force flag support.
- [x] 5.2 Update any command summary documentation that lists supported worktree subcommands.
- [x] 5.3 Run focused CLI parser and worktree integration tests.
- [x] 5.4 Run the full test suite if focused tests pass.
- [x] 5.5 Run `nix develop -c openspec validate worktree-move-command --strict`.
