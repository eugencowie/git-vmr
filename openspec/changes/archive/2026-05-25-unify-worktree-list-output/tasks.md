## 1. Renderer Model

- [x] 1.1 Refactor `src/commands/worktree.rs` list rendering so each aggregate root is rendered as one unified entry.
- [x] 1.2 Group branch-backed child worktree entries by branch name instead of `(branch, short HEAD)`.
- [x] 1.3 Keep detached child worktree entries grouped by short HEAD and repository coverage.
- [x] 1.4 Preserve deterministic ordering for aggregate roots, state summaries, and repository suffixes.

## 2. Output Coverage

- [x] 2.1 Add or update focused renderer tests for a shared branch with different child HEAD hashes rendering once.
- [x] 2.2 Add or update command tests for mixed branch state rendering under one aggregate root.
- [x] 2.3 Add or update command tests for detached child state rendering under one aggregate root.
- [x] 2.4 Update partial aggregate coverage assertions to expect one aggregate-root entry with repository suffixes on the relevant state.

## 3. Documentation and Verification

- [x] 3.1 Update worktree list documentation or examples if they describe VMR-specific list output.
- [x] 3.2 Run the focused worktree test suite.
- [x] 3.3 Run `openspec status --change unify-worktree-list-output` and confirm the change is apply-ready.
