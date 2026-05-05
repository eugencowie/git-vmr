# rm-command Specification

## Purpose
Define `git vmr rm` behavior for removing tracked paths across child repositories from any working directory inside a virtual monorepo, including path routing, recursive removal, root expansion, validation, and Git failure reporting.

## Requirements
### Requirement: Rm removes tracked paths across child repositories
The `git vmr rm <path>...` command SHALL discover the VMR root from the effective working directory, route each path argument to its owning immediate child Git repository, and remove the corresponding repo-relative tracked path with Git.

#### Scenario: Remove a file from the VMR root
- **WHEN** user runs `git vmr rm backend/src/main.rs` from a VMR root containing a `backend` child Git repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

#### Scenario: Remove files in multiple repositories
- **WHEN** user runs `git vmr rm backend/src/main.rs frontend/src/app.rs` from the VMR root
- **THEN** the command SHALL remove `src/main.rs` in `backend` and `src/app.rs` in `frontend`

#### Scenario: Git rm semantics are preserved
- **WHEN** user runs `git vmr rm backend/untracked.rs` for a path that is not tracked by Git
- **THEN** the command SHALL fail with the underlying Git removal error and SHALL include the child repository path in the error message

### Requirement: Rm interprets paths relative to the effective working directory
The `git vmr rm` command SHALL interpret relative path arguments against the resolved working directory from cwd or the global `-C <path>` flag.

#### Scenario: Remove a sibling repository path from inside a child repo
- **WHEN** user runs `git vmr rm ../backend/src/main.rs` from inside the `frontend` child repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

#### Scenario: Remove with -C working directory
- **WHEN** user runs `git vmr -C frontend rm ../backend/src/main.rs` from the VMR root
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

#### Scenario: Remove current subtree path inside a child repo
- **WHEN** user runs `git vmr rm src/main.rs` from inside the `frontend` repository
- **THEN** the command SHALL remove `src/main.rs` from the `frontend` working tree and stage the deletion in `frontend`

### Requirement: Rm supports recursive removal only with short recursive flag
The `git vmr rm` command SHALL support `-r` to remove tracked directories or repository roots recursively. Without the recursive flag, directory and repository-root removals SHALL fail without recursively removing tracked files.

#### Scenario: Recursive flag removes a child repository directory
- **WHEN** user runs `git vmr rm -r backend/src` from the VMR root and `src` is a tracked directory in `backend`
- **THEN** the command SHALL remove `src` recursively from the `backend` working tree and stage the deletions in `backend`

#### Scenario: Directory removal without recursive flag fails
- **WHEN** user runs `git vmr rm backend/src` from the VMR root and `src` is a tracked directory in `backend`
- **THEN** the command SHALL fail without recursively removing tracked files

### Requirement: Rm requires recursive flag for VMR root expansion
When a path argument resolves to the VMR root itself, the `git vmr rm` command SHALL require `-r`. With recursive intent, the command SHALL remove `.` recursively in every immediate child Git repository and SHALL skip non-Git child directories.

#### Scenario: Reject VMR root removal without recursive flag
- **WHEN** user runs `git vmr rm .` from the VMR root
- **THEN** the command SHALL fail before invoking any `git rm` operation

#### Scenario: Recursively remove all repositories from VMR root
- **WHEN** user runs `git vmr rm -r .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL remove `.` recursively in both `backend` and `frontend`

#### Scenario: VMR root expansion skips non-Git directories
- **WHEN** user runs `git vmr rm -r .` from a VMR root containing a child Git repository `backend` and a non-Git directory `docs`
- **THEN** the command SHALL remove `.` recursively in `backend` and SHALL NOT fail because of `docs`

### Requirement: Rm rejects invalid path ownership before removal
The `git vmr rm` command SHALL validate all path arguments before invoking any `git rm` operation. It SHALL fail without removing files when any path is outside the VMR root, inside `.gitvmr/`, or not owned by an immediate child Git repository.

#### Scenario: Reject path outside the VMR
- **WHEN** user runs `git vmr rm ../outside.txt` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without removing any files

#### Scenario: Reject VMR metadata path
- **WHEN** user runs `git vmr rm .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without removing any files

#### Scenario: Reject explicit non-Git child directory
- **WHEN** user runs `git vmr rm docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without removing any files

#### Scenario: Reject file directly under VMR root
- **WHEN** user runs `git vmr rm README.md` and `README.md` is directly under the VMR root rather than inside a child Git repository
- **THEN** the command SHALL fail without removing any files

### Requirement: Rm surfaces Git removal failures
If any underlying `git rm` invocation fails after path validation succeeds, the `git vmr rm` command SHALL exit with a non-zero status and report the failing repository and Git error.

#### Scenario: Git rm fails in a repository
- **WHEN** a path routes to a child repository but the underlying `git rm` command fails
- **THEN** the command SHALL fail and include the child repository path in the error message
