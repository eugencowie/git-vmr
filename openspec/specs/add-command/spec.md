# add-command Specification

## Purpose
Define `git vmr add` behavior for staging paths across child repositories from any working directory inside a virtual monorepo, including path routing, validation, root expansion, and Git failure reporting.
## Requirements
### Requirement: Add stages paths across child repositories
The `git vmr add <path>...` command SHALL discover the VMR root from the effective working directory, route each path argument to its owning immediate child Git repository, and stage the corresponding repo-relative path with Git.

#### Scenario: Stage a file from the VMR root
- **WHEN** user runs `git vmr add backend/src/main.rs` from a VMR root containing a `backend` child Git repository
- **THEN** the command SHALL stage `src/main.rs` in the `backend` repository

#### Scenario: Stage files in multiple repositories
- **WHEN** user runs `git vmr add backend/src/main.rs frontend/src/app.rs` from the VMR root
- **THEN** the command SHALL stage `src/main.rs` in `backend` and `src/app.rs` in `frontend`

#### Scenario: Stage a child repository directory
- **WHEN** user runs `git vmr add backend` from the VMR root
- **THEN** the command SHALL stage `.` in the `backend` repository

### Requirement: Add interprets paths relative to the effective working directory
The `git vmr add` command SHALL interpret relative path arguments against the resolved working directory from cwd or the global `-C <path>` flag.

#### Scenario: Stage a sibling repository path from inside a child repo
- **WHEN** user runs `git vmr add ../backend/src/main.rs` from inside the `frontend` child repository
- **THEN** the command SHALL stage `src/main.rs` in the `backend` repository

#### Scenario: Stage with -C working directory
- **WHEN** user runs `git vmr -C frontend add ../backend/src/main.rs` from the VMR root
- **THEN** the command SHALL stage `src/main.rs` in the `backend` repository

#### Scenario: Stage current subtree inside a child repo
- **WHEN** user runs `git vmr add .` from inside `frontend/src`
- **THEN** the command SHALL stage `src` in the `frontend` repository

### Requirement: Add supports deleted and untracked paths
The `git vmr add` command SHALL support staging paths whether they currently exist on disk or have been deleted, provided the path can be routed lexically to an owning child Git repository.

#### Scenario: Stage an untracked file
- **WHEN** user runs `git vmr add backend/new.rs` for an untracked file in the `backend` repository
- **THEN** the command SHALL stage `new.rs` as an added file in `backend`

#### Scenario: Stage a deleted file
- **WHEN** user runs `git vmr add backend/old.rs` after `old.rs` was deleted from the `backend` repository
- **THEN** the command SHALL stage the deletion of `old.rs` in `backend`

### Requirement: Add expands the VMR root to all child Git repositories
When a path argument resolves to the VMR root itself, the `git vmr add` command SHALL stage `.` in every immediate child Git repository and SHALL skip non-Git child directories.

#### Scenario: Stage all repositories from VMR root
- **WHEN** user runs `git vmr add .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL stage `.` in both `backend` and `frontend`

#### Scenario: Root expansion skips non-Git directories
- **WHEN** user runs `git vmr add .` from a VMR root containing a child Git repository `backend` and a non-Git directory `docs`
- **THEN** the command SHALL stage `.` in `backend` and SHALL NOT fail because of `docs`

### Requirement: Add rejects invalid path ownership before staging
The `git vmr add` command SHALL validate all path arguments before invoking any `git add` operation. It SHALL fail without staging files when any path is outside the VMR root, inside `.gitvmr/`, or not owned by an immediate child Git repository.

#### Scenario: Reject path outside the VMR
- **WHEN** user runs `git vmr add ../outside.txt` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without staging any files

#### Scenario: Reject VMR metadata path
- **WHEN** user runs `git vmr add .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without staging any files

#### Scenario: Reject explicit non-Git child directory
- **WHEN** user runs `git vmr add docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without staging any files

#### Scenario: Reject file directly under VMR root
- **WHEN** user runs `git vmr add README.md` and `README.md` is directly under the VMR root rather than inside a child Git repository
- **THEN** the command SHALL fail without staging any files

### Requirement: Add surfaces Git staging failures
If any underlying `git add` invocation fails after path validation succeeds, the `git vmr add` command SHALL exit with a non-zero status and report the failing repository and Git error.

#### Scenario: Git add fails in a repository
- **WHEN** a path routes to a child repository but the underlying `git add` command fails
- **THEN** the command SHALL fail and include the child repository path in the error message

### Requirement: Add supports all-mode staging
The `git vmr add` command SHALL accept `-A` and `--all` to stage additions, modifications, and removals using Git all-mode semantics in each affected child repository.

#### Scenario: Stage all repositories without pathspecs
- **WHEN** user runs `git vmr add -A` from inside a VMR containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL stage `.` with `-A` in both `backend` and `frontend`

#### Scenario: Stage all repositories with long flag
- **WHEN** user runs `git vmr add --all` from inside a VMR containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL stage `.` with `--all` in both `backend` and `frontend`

#### Scenario: Stage routed pathspecs with all-mode
- **WHEN** user runs `git vmr add -A backend/src frontend/app` from the VMR root
- **THEN** the command SHALL stage `src` with `-A` in `backend` and `app` with `-A` in `frontend`

#### Scenario: Reject no pathspecs without all-mode
- **WHEN** user runs `git vmr add` without `-A`, `--all`, or pathspecs
- **THEN** the command SHALL fail during argument parsing without staging any child repository

### Requirement: Add supports force staging ignored paths
The `git vmr add` command SHALL accept `-f` and `--force` and SHALL pass the force option to each underlying `git add` invocation after VMR path routing succeeds.

#### Scenario: Stage ignored file with short force flag
- **WHEN** user runs `git vmr add -f backend/generated.log` and `generated.log` is ignored by the `backend` repository
- **THEN** the command SHALL stage `generated.log` in `backend`

#### Scenario: Stage ignored file with long force flag
- **WHEN** user runs `git vmr add --force backend/generated.log` and `generated.log` is ignored by the `backend` repository
- **THEN** the command SHALL stage `generated.log` in `backend`

### Requirement: Add supports chmod executable-bit updates
The `git vmr add` command SHALL accept `--chmod=+x` and `--chmod=-x` and SHALL pass the chmod option to each underlying `git add` invocation after VMR path routing succeeds.

#### Scenario: Set executable bit in index
- **WHEN** user runs `git vmr add --chmod=+x backend/script.sh`
- **THEN** the command SHALL set the executable bit for `script.sh` in the `backend` repository index without requiring the working tree file mode to change

#### Scenario: Clear executable bit in index
- **WHEN** user runs `git vmr add --chmod=-x backend/script.sh`
- **THEN** the command SHALL clear the executable bit for `script.sh` in the `backend` repository index without requiring the working tree file mode to change

#### Scenario: Reject unsupported chmod value
- **WHEN** user runs `git vmr add --chmod=bad backend/script.sh`
- **THEN** the command SHALL fail during argument parsing without staging any child repository

### Requirement: Add flags preserve VMR path validation
The `git vmr add` command SHALL validate VMR ownership for all explicit pathspecs before invoking any `git add` operation, even when `-A`, `--all`, `-f`, `--force`, or `--chmod` is present.

#### Scenario: Force does not allow VMR metadata path
- **WHEN** user runs `git vmr add --force .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without staging any child repository

#### Scenario: All-mode with invalid pathspec fails before staging
- **WHEN** user runs `git vmr add -A backend/src docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without staging any child repository

#### Scenario: Chmod does not allow path outside VMR
- **WHEN** user runs `git vmr add --chmod=+x ../outside.sh` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without staging any child repository

