# mv-command Specification

## Purpose
Define `git vmr mv` behavior for moving tracked paths within and across child repositories from any working directory inside a virtual monorepo, including path routing, destination-directory semantics, validation, staging, and failure reporting.

## Requirements

### Requirement: Mv moves paths within a child repository
The `git vmr mv <source> <destination>` command SHALL discover the VMR root from the effective working directory, route both operands to their owning immediate child Git repositories, and move paths within a single child repository using Git move semantics.

#### Scenario: Move a file within one repository from the VMR root
- **WHEN** user runs `git vmr mv backend/src/old.rs backend/src/new.rs` from a VMR root containing a `backend` child Git repository
- **THEN** the command SHALL move `src/old.rs` to `src/new.rs` in the `backend` working tree and stage the move in `backend`

#### Scenario: Move a directory within one repository
- **WHEN** user runs `git vmr mv backend/old-dir backend/new-dir` from the VMR root and `old-dir` is tracked in `backend`
- **THEN** the command SHALL move `old-dir` to `new-dir` in the `backend` working tree and stage the move in `backend`

#### Scenario: Preserve Git failure for an untracked source
- **WHEN** user runs `git vmr mv backend/untracked.rs backend/moved.rs` for a source path that is not tracked by Git
- **THEN** the command SHALL fail with the underlying Git move error and SHALL include the child repository path in the error message

### Requirement: Mv moves paths between child repositories
The `git vmr mv <source> <destination>` command SHALL support moving a path from one child Git repository to another by moving the worktree path, staging the source deletion in the source repository, and staging the destination addition in the destination repository.

#### Scenario: Move a file between repositories
- **WHEN** user runs `git vmr mv backend/src/shared.rs frontend/src/shared.rs` from a VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL remove `src/shared.rs` from `backend`, create `src/shared.rs` in `frontend`, stage the deletion in `backend`, and stage the addition in `frontend`

#### Scenario: Move a directory between repositories
- **WHEN** user runs `git vmr mv backend/shared frontend/shared` from the VMR root and `shared` is a tracked directory in `backend`
- **THEN** the command SHALL move the directory tree to `frontend/shared`, stage the deletions in `backend`, and stage the additions in `frontend`

#### Scenario: Reject cross-repository move for untracked source before moving
- **WHEN** user runs `git vmr mv backend/untracked.rs frontend/untracked.rs` for a source path that is not tracked by Git
- **THEN** the command SHALL fail without moving the file and SHALL include the source child repository path in the error message

### Requirement: Mv interprets paths relative to the effective working directory
The `git vmr mv` command SHALL interpret both source and destination operands against the resolved working directory from cwd or the global `-C <path>` flag.

#### Scenario: Move a sibling repository path from inside a child repo
- **WHEN** user runs `git vmr mv ../backend/src/old.rs ../backend/src/new.rs` from inside the `frontend` child repository
- **THEN** the command SHALL move `src/old.rs` to `src/new.rs` in the `backend` repository

#### Scenario: Move with -C working directory
- **WHEN** user runs `git vmr -C frontend mv ../backend/src/old.rs ../backend/src/new.rs` from the VMR root
- **THEN** the command SHALL move `src/old.rs` to `src/new.rs` in the `backend` repository

#### Scenario: Move current repository path from inside a child repo
- **WHEN** user runs `git vmr mv src/old.rs src/new.rs` from inside the `frontend` repository
- **THEN** the command SHALL move `src/old.rs` to `src/new.rs` in the `frontend` repository

### Requirement: Mv supports destination-directory semantics
When the destination operand resolves to an existing directory inside a child Git repository, the `git vmr mv` command SHALL move the source path under that directory using the source basename.

#### Scenario: Move file into an existing directory in the same repository
- **WHEN** user runs `git vmr mv backend/src/main.rs backend/archive` from the VMR root and `archive` is an existing directory in `backend`
- **THEN** the command SHALL move `src/main.rs` to `archive/main.rs` in `backend`

#### Scenario: Move file into another child repository root
- **WHEN** user runs `git vmr mv backend/src/shared.rs frontend` from the VMR root
- **THEN** the command SHALL move `src/shared.rs` to `shared.rs` in the `frontend` repository, stage the deletion in `backend`, and stage the addition in `frontend`

#### Scenario: Missing destination parent fails
- **WHEN** user runs `git vmr mv backend/src/main.rs frontend/missing/main.rs` and `frontend/missing` does not exist
- **THEN** the command SHALL fail without moving the source path

### Requirement: Mv moves multiple sources into a destination directory
The `git vmr mv <source>... <destination-directory>` command SHALL support two or more source operands followed by one destination directory operand. It SHALL discover the VMR root from the effective working directory, route every source and the destination directory to their owning immediate child Git repositories, and move each source under the destination directory using the source basename.

#### Scenario: Move multiple files within one repository
- **WHEN** user runs `git vmr mv backend/src/a.rs backend/src/b.rs backend/archive` from a VMR root containing a `backend` child Git repository and `archive` is an existing directory in `backend`
- **THEN** the command SHALL move `src/a.rs` to `archive/a.rs` in `backend`
- **AND** the command SHALL move `src/b.rs` to `archive/b.rs` in `backend`
- **AND** the command SHALL stage both moves in `backend`

#### Scenario: Move multiple files from one repository into another repository root
- **WHEN** user runs `git vmr mv backend/src/a.rs backend/src/b.rs frontend` from a VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL remove `src/a.rs` and `src/b.rs` from `backend`
- **AND** the command SHALL create `a.rs` and `b.rs` in `frontend`
- **AND** the command SHALL stage the deletions in `backend`
- **AND** the command SHALL stage the additions in `frontend`

#### Scenario: Move multiple files from multiple repositories into one destination directory
- **WHEN** user runs `git vmr mv backend/src/a.rs frontend/src/b.rs tools/archive` from a VMR root containing `backend`, `frontend`, and `tools` child Git repositories and `archive` is an existing directory in `tools`
- **THEN** the command SHALL remove `src/a.rs` from `backend`
- **AND** the command SHALL remove `src/b.rs` from `frontend`
- **AND** the command SHALL create `archive/a.rs` and `archive/b.rs` in `tools`
- **AND** the command SHALL stage the deletion in each source repository and the additions in `tools`

#### Scenario: Multi-source move uses effective working directory
- **WHEN** user runs `git vmr mv ../backend/src/a.rs ../backend/src/b.rs ../frontend/archive` from inside the `tools` child repository and `archive` is an existing directory in `frontend`
- **THEN** the command SHALL move `src/a.rs` and `src/b.rs` from `backend` to `archive/a.rs` and `archive/b.rs` in `frontend`

### Requirement: Mv validates multi-source moves before mutation
The `git vmr mv <source>... <destination-directory>` command SHALL validate all sources and the destination before invoking any move operation. It SHALL fail without moving files when any operand has invalid VMR ownership, any source is untracked, the destination operand is not an existing directory, any final destination already exists, any source lacks a basename, or two sources would produce the same final destination.

#### Scenario: Reject multi-source move when destination is not an existing directory
- **WHEN** user runs `git vmr mv backend/src/a.rs backend/src/b.rs frontend/archive` and `frontend/archive` does not exist
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject multi-source move with destination file
- **WHEN** user runs `git vmr mv backend/src/a.rs backend/src/b.rs frontend/existing.rs` and `frontend/existing.rs` is an existing file
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject duplicate final destinations
- **WHEN** user runs `git vmr mv backend/src/lib.rs frontend/src/lib.rs tools/archive` and `tools/archive` is an existing directory
- **THEN** the command SHALL fail because both sources would move to `archive/lib.rs`
- **AND** the command SHALL fail without moving any files

#### Scenario: Reject existing final destination
- **WHEN** user runs `git vmr mv backend/src/a.rs frontend/src/b.rs tools/archive` and `tools/archive/a.rs` already exists
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject invalid source before moving any source
- **WHEN** user runs `git vmr mv backend/src/a.rs docs/b.rs tools/archive` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject untracked source before moving any source
- **WHEN** user runs `git vmr mv backend/src/a.rs frontend/untracked.rs tools/archive` and `frontend/untracked.rs` is not tracked by Git
- **THEN** the command SHALL fail without moving any files

### Requirement: Mv rejects invalid path ownership before moving
The `git vmr mv` command SHALL validate both operands before invoking any move operation. It SHALL fail without moving files when either operand is outside the VMR root, resolves to the VMR root aggregate, is inside `.gitvmr/`, or is not owned by an immediate child Git repository.

#### Scenario: Reject source outside the VMR
- **WHEN** user runs `git vmr mv ../outside.txt backend/outside.txt` and the resolved source path is outside the VMR root
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject destination outside the VMR
- **WHEN** user runs `git vmr mv backend/src/main.rs ../outside.txt` and the resolved destination path is outside the VMR root
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject VMR metadata path
- **WHEN** user runs `git vmr mv backend/src/main.rs .gitvmr/main.rs` from the VMR root
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject explicit non-Git child directory
- **WHEN** user runs `git vmr mv backend/src/main.rs docs/main.rs` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject file directly under VMR root
- **WHEN** user runs `git vmr mv backend/src/main.rs README.md` and `README.md` is directly under the VMR root rather than inside a child Git repository
- **THEN** the command SHALL fail without moving any files

#### Scenario: Reject VMR root aggregate operand
- **WHEN** user runs `git vmr mv . backend/all` from the VMR root
- **THEN** the command SHALL fail without moving any files

### Requirement: Mv surfaces move and staging failures
If any underlying Git or filesystem operation fails after validation succeeds, the `git vmr mv` command SHALL exit with a non-zero status and report the failing repository or path context.

#### Scenario: Git move fails in a repository
- **WHEN** source and destination route to the same child repository but the underlying `git mv` command fails
- **THEN** the command SHALL fail and include the child repository path in the error message

#### Scenario: Cross-repository staging fails
- **WHEN** a cross-repository move succeeds in the filesystem but staging fails in either child repository
- **THEN** the command SHALL fail and include the failing child repository path in the error message
