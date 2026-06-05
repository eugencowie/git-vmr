## ADDED Requirements

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
