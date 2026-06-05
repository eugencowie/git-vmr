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

### Requirement: Rm supports force removal
The `git vmr rm` command SHALL accept `-f` and `--force` and SHALL pass force removal to each underlying `git rm` invocation after VMR path routing succeeds.

#### Scenario: Remove modified tracked file with short force flag
- **WHEN** user runs `git vmr rm -f backend/src/main.rs` and `src/main.rs` has working tree modifications in the `backend` repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

#### Scenario: Remove modified tracked file with long force flag
- **WHEN** user runs `git vmr rm --force backend/src/main.rs` and `src/main.rs` has working tree modifications in the `backend` repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

### Requirement: Rm supports dry-run previews
The `git vmr rm` command SHALL accept `-n` and `--dry-run` and SHALL pass dry-run mode to each underlying `git rm` invocation after VMR path routing succeeds. In dry-run mode, the command SHALL report Git's preview output and SHALL NOT remove files from child repository working trees or indexes.

#### Scenario: Preview file removal with short dry-run flag
- **WHEN** user runs `git vmr rm -n backend/src/main.rs`
- **THEN** the command SHALL report that `src/main.rs` would be removed from `backend`
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree
- **AND** the deletion SHALL NOT be staged in `backend`

#### Scenario: Preview file removal with long dry-run flag
- **WHEN** user runs `git vmr rm --dry-run backend/src/main.rs`
- **THEN** the command SHALL report that `src/main.rs` would be removed from `backend`
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree
- **AND** the deletion SHALL NOT be staged in `backend`

#### Scenario: Preview VMR root removal still requires recursive flag
- **WHEN** user runs `git vmr rm -n .` from the VMR root
- **THEN** the command SHALL fail before invoking any `git rm` operation

#### Scenario: Preview recursive VMR root removal
- **WHEN** user runs `git vmr rm -n -r .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL report removals that would occur in `backend` and `frontend`
- **AND** no deletion SHALL be staged in either child repository
- **AND** no tracked file SHALL be removed from either child repository working tree

### Requirement: Rm supports cached index-only removal
The `git vmr rm` command SHALL accept `--cached` and SHALL pass cached mode to each underlying `git rm` invocation after VMR path routing succeeds. In cached mode, the command SHALL remove routed paths from child repository indexes while leaving working tree files in place.

#### Scenario: Remove tracked file from index only
- **WHEN** user runs `git vmr rm --cached backend/src/main.rs`
- **THEN** the command SHALL stage removal of `src/main.rs` from the `backend` index
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree

#### Scenario: Remove tracked directory from index only with recursive flag
- **WHEN** user runs `git vmr rm --cached -r backend/src`
- **THEN** the command SHALL stage removal of tracked paths under `src` from the `backend` index
- **AND** the files under `src` SHALL remain present in the `backend` working tree

### Requirement: Rm flags preserve VMR path validation
The `git vmr rm` command SHALL validate VMR ownership for all explicit pathspecs before invoking any `git rm` operation, even when `-f`, `--force`, `-n`, `--dry-run`, or `--cached` is present.

#### Scenario: Force does not allow VMR metadata path
- **WHEN** user runs `git vmr rm --force .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without removing files from any child repository

#### Scenario: Dry-run does not allow explicit non-Git child path
- **WHEN** user runs `git vmr rm --dry-run docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without invoking any `git rm` operation

#### Scenario: Cached mode does not allow path outside VMR
- **WHEN** user runs `git vmr rm --cached ../outside.txt` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without removing files from any child repository index
