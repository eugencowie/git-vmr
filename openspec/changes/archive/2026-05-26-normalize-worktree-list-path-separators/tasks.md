## 1. Rendering

- [x] 1.1 Add or reuse a Git-style path display helper for worktree list rendering.
- [x] 1.2 Update `git vmr worktree list` aggregate root rendering to use the helper instead of direct `Path::display()` output.
- [x] 1.3 Preserve existing multi-line state rendering, repository suffix rendering, and deterministic ordering behavior.

## 2. Tests

- [x] 2.1 Add host-independent coverage that renders a Windows-style aggregate path such as `C:\Projects\vmr` as `C:/Projects/vmr`.
- [x] 2.2 Add coverage for mixed separator inputs so `C:\Projects\vmr` and `C:/Worktrees/new-feature` both render with `/` separators.
- [x] 2.3 Ensure existing worktree list integration tests still cover grouping, filtering, branch, detached HEAD, and sorting behavior.

## 3. Verification

- [x] 3.1 Run the focused worktree command tests.
- [x] 3.2 Run the relevant OpenSpec validation for `normalize-worktree-list-path-separators`.
