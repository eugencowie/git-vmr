## 1. CLI Shape

- [x] 1.1 Add `List` to `WorktreeCommand` and dispatch it to the worktree command module.
- [x] 1.2 Add parser tests for `git vmr worktree list`, rejected extra operands, and rejected list options.
- [x] 1.3 Update the existing unsupported-subcommand parser expectation so `list` is accepted and another unsupported subcommand is rejected.

## 2. Git Worktree List Model

- [x] 2.1 Add internal data structures for child worktree list entries, including path, HEAD hash, branch or detached state, and optional metadata flags.
- [x] 2.2 Implement `git worktree list --porcelain -z` invocation for one child repository.
- [x] 2.3 Parse porcelain worktree records, tolerate unknown fields, and report malformed or unreadable output with repository-specific context.
- [x] 2.4 Add focused unit tests for porcelain parsing, including branch, detached HEAD, locked, prunable, and unknown fields.

## 3. Aggregate Listing

- [x] 3.1 Implement `commands::worktree::list` to discover the VMR root and immediate child Git repositories.
- [x] 3.2 Group child worktree entries by aggregate root path, including the main VMR root and marked linked aggregate roots.
- [x] 3.3 Filter out unmarked matching parents and arbitrary child worktree paths that do not match the aggregate layout.
- [x] 3.4 Render deterministic aggregate output sorted by path, grouping shared branch or detached states and appending repository suffixes for partial or mixed state.
- [x] 3.5 Fail clearly when any child repository worktree list cannot be read.

## 4. Integration Tests

- [x] 4.1 Test listing the main aggregate worktree for multiple child repositories.
- [x] 4.2 Test listing a linked aggregate worktree created by `git vmr worktree add`.
- [x] 4.3 Test non-Git child directories are skipped.
- [x] 4.4 Test unmarked and arbitrary child worktrees are omitted.
- [x] 4.5 Test mixed branches, detached HEAD state, partial aggregate coverage, deterministic path order, and sorted repository suffixes.
- [x] 4.6 Test `git vmr worktree list` from inside a child repository and through global `-C`.

## 5. Documentation and Validation

- [x] 5.1 Update `docs/git-vmr/worktree.md` to mark `list` as supported and describe the aggregate listing behavior.
- [x] 5.2 Run `cargo fmt`.
- [x] 5.3 Run the relevant test suite, including `cargo test worktree` and parser tests.
- [x] 5.4 Run `openspec validate worktree-list-command --strict`.
