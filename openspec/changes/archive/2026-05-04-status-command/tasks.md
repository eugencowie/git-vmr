## 1. Dependencies

- [x] 1.1 Add `gix` dependency to `Cargo.toml` (minimal features: status, worktree — no network/remote)
- [x] 1.2 Add `rayon` dependency to `Cargo.toml`
- [x] 1.3 Add `anstyle` and `anstream` dependencies to `Cargo.toml`

## 2. VMR Root Discovery

- [x] 2.1 Create `src/config/vmr.rs` with `find_vmr_root(start: &Path) -> Result<PathBuf>` that walks ancestors looking for `.gitvmr/`
- [x] 2.2 Register the new module in `src/config/mod.rs`
- [x] 2.3 Write unit tests for VMR root discovery: found in start dir, found in ancestor, not found at all
- [x] 2.4 Write integration test: `git vmr status` fails with error when run outside a VMR

## 3. Status Collection

- [x] 3.1 Create `src/cli/status.rs` with a `RepoStatus` struct holding branch name (or detached HEAD info), initial commit flag, staged changes, unstaged changes, and untracked files
- [x] 3.2 Implement `collect_statuses(vmr_root: &Path) -> Result<Vec<(String, RepoStatus)>>` that lists immediate child dirs, opens each with `gix`, skips non-git dirs, collects status via `rayon::par_iter`, fails on real errors
- [x] 3.3 Implement detached HEAD detection: when repo is in detached state, store short commit hash
- [x] 3.4 Implement no-commits detection: when repo has no commits, flag as initial
- [x] 3.5 Write unit tests for status collection with temp dirs containing git repos

## 4. Output Rendering

- [x] 4.1 Implement `render_status(statuses: &[(String, RepoStatus)], working_dir: &Path, vmr_root: &Path)` that groups repos by branch name and renders unified output
- [x] 4.2 Implement branch grouping: single branch omits repo list, multiple branches show repo names in parentheses
- [x] 4.3 Implement detached HEAD rendering: `HEAD detached at <hash> (<repo-name>)`
- [x] 4.4 Implement no-commits rendering: `On branch <name> (<repo-name>)` + `No commits yet.`
- [x] 4.5 Implement cwd-relative path computation for all file paths
- [x] 4.6 Implement colored file entries using `anstyle` (green for staged, red for unstaged/untracked) with `anstream` for tty detection; headers remain uncolored
- [x] 4.7 Write unit tests for rendering: same branch, diverged branches, detached HEAD, no commits, path relativization, colored vs uncolored output

## 5. CLI Integration

- [x] 5.1 Add `Status` variant to the `Command` enum in `src/cli/mod.rs` with doc comment
- [x] 5.2 Register `src/cli/status.rs` module in `src/cli/mod.rs`
- [x] 5.3 Wire the `Status` command to call `find_vmr_root`, then `collect_statuses`, then `render_status`
- [x] 5.4 Write integration tests: status in VMR root, status from nested directory via `-C`, status with no child repos, status with corrupted git dir

## 6. Build Verification

- [x] 6.1 Run `cargo build` and resolve any compilation errors
- [x] 6.2 Run `cargo test` and ensure all tests pass
- [x] 6.3 Run `cargo fmt --check` and fix any formatting issues
- [x] 6.4 Run `cargo clippy` and resolve any warnings
