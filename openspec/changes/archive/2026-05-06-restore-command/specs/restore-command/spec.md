## ADDED Requirements

### Requirement: Restore discards unstaged tracked changes across child repositories
The `git vmr restore <pathspec>...` command SHALL discover the VMR root from the effective working directory, route each path argument to its owning immediate child Git repository, and restore the corresponding repo-relative path in the working tree using Git restore semantics.

#### Scenario: Restore modified file from the VMR root
- **WHEN** user runs `git vmr restore backend/src/main.rs` from a VMR root containing a `backend` child Git repository and `src/main.rs` has unstaged modifications
- **THEN** the command SHALL discard the unstaged working tree modifications to `src/main.rs` in the `backend` repository

#### Scenario: Restore deleted file from the VMR root
- **WHEN** user runs `git vmr restore backend/src/main.rs` after `src/main.rs` was deleted from the `backend` working tree
- **THEN** the command SHALL restore `src/main.rs` in the `backend` working tree

#### Scenario: Restore paths in multiple repositories
- **WHEN** user runs `git vmr restore backend/src/main.rs frontend/src/app.rs` from the VMR root
- **THEN** the command SHALL restore `src/main.rs` in `backend` and `src/app.rs` in `frontend`

#### Scenario: Preserve Git failure for untracked file
- **WHEN** user runs `git vmr restore backend/untracked.rs` for an untracked path in the `backend` repository
- **THEN** the command SHALL fail with the underlying Git restore error and SHALL include the child repository path in the error message

### Requirement: Restore supports staged and worktree targets
The `git vmr restore` command SHALL support `--staged` and `--worktree` target flags and SHALL pass the selected targets to Git restore for each routed child repository.

#### Scenario: Staged restore unstages a modified file
- **WHEN** user runs `git vmr restore --staged backend/src/main.rs` after `src/main.rs` has staged modifications in the `backend` repository
- **THEN** the command SHALL unstage `src/main.rs` in `backend` without discarding working tree modifications

#### Scenario: Worktree flag restores working tree changes
- **WHEN** user runs `git vmr restore --worktree backend/src/main.rs` after `src/main.rs` has unstaged modifications in the `backend` repository
- **THEN** the command SHALL discard the unstaged working tree modifications to `src/main.rs` in `backend`

#### Scenario: Combined staged and worktree restore resets both targets
- **WHEN** user runs `git vmr restore --staged --worktree backend/src/main.rs` after `src/main.rs` has staged and working tree modifications in the `backend` repository
- **THEN** the command SHALL restore both the index and working tree state for `src/main.rs` in `backend` using Git restore semantics

#### Scenario: Restore does not support source option
- **WHEN** user runs `git vmr restore --source HEAD~1 backend/src/main.rs`
- **THEN** the command SHALL fail during argument parsing without invoking any Git restore operation

### Requirement: Restore interprets paths relative to the effective working directory
The `git vmr restore` command SHALL interpret relative path arguments against the resolved working directory from cwd or the global `-C <path>` flag.

#### Scenario: Restore sibling repository path from inside a child repo
- **WHEN** user runs `git vmr restore ../backend/src/main.rs` from inside the `frontend` child repository
- **THEN** the command SHALL restore `src/main.rs` in the `backend` repository

#### Scenario: Restore with -C working directory
- **WHEN** user runs `git vmr -C frontend restore ../backend/src/main.rs` from the VMR root
- **THEN** the command SHALL restore `src/main.rs` in the `backend` repository

#### Scenario: Restore current subtree inside a child repo
- **WHEN** user runs `git vmr restore .` from inside `frontend/src`
- **THEN** the command SHALL restore `src` in the `frontend` repository

### Requirement: Restore expands the VMR root to all child Git repositories
When a path argument resolves to the VMR root itself, the `git vmr restore` command SHALL restore `.` in every immediate child Git repository and SHALL skip non-Git child directories.

#### Scenario: Restore all repositories from VMR root
- **WHEN** user runs `git vmr restore .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL restore `.` in both `backend` and `frontend`

#### Scenario: Staged restore all repositories from VMR root
- **WHEN** user runs `git vmr restore --staged .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL restore the staged state for `.` in both `backend` and `frontend`

#### Scenario: Root expansion skips non-Git directories
- **WHEN** user runs `git vmr restore .` from a VMR root containing a child Git repository `backend` and a non-Git directory `docs`
- **THEN** the command SHALL restore `.` in `backend` and SHALL NOT fail because of `docs`

### Requirement: Restore rejects invalid path ownership before mutation
The `git vmr restore` command SHALL validate all path arguments before invoking any `git restore` operation. It SHALL fail without restoring files when any path is outside the VMR root, inside `.gitvmr/`, or not owned by an immediate child Git repository.

#### Scenario: Reject path outside the VMR
- **WHEN** user runs `git vmr restore ../outside.txt` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without restoring any files

#### Scenario: Reject VMR metadata path
- **WHEN** user runs `git vmr restore .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without restoring any files

#### Scenario: Reject explicit non-Git child directory
- **WHEN** user runs `git vmr restore docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without restoring any files

#### Scenario: Reject file directly under VMR root
- **WHEN** user runs `git vmr restore README.md` and `README.md` is directly under the VMR root rather than inside a child Git repository
- **THEN** the command SHALL fail without restoring any files

### Requirement: Restore surfaces Git restore failures
If any underlying `git restore` invocation fails after path validation succeeds, the `git vmr restore` command SHALL exit with a non-zero status and report the failing repository and Git error.

#### Scenario: Git restore fails in a repository
- **WHEN** a path routes to a child repository but the underlying `git restore` command fails
- **THEN** the command SHALL fail and include the child repository path in the error message
