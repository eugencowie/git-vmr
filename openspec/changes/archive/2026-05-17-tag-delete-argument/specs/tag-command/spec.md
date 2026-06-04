## ADDED Requirements

### Requirement: Tag command deletes local tags across child repositories
When `-d` or `--delete` is provided with a tag name, the `git vmr tag` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to delete the named local tag in every immediate child Git repository. Tag deletion attempts SHALL be independent: a failure in one child repository SHALL NOT prevent tag deletion from being attempted in other discovered child Git repositories.

#### Scenario: Delete tag in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** local tag `v1.0.0` exists in both repositories
- **AND** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL delete local tag `v1.0.0` in both repositories using Git's tag deletion semantics
- **AND** the command SHALL succeed

#### Scenario: Long delete flag deletes tag in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** local tag `v1.0.0` exists in both repositories
- **AND** user runs `git vmr tag --delete v1.0.0`
- **THEN** the command SHALL delete local tag `v1.0.0` in both repositories using Git's tag deletion semantics
- **AND** the command SHALL succeed

#### Scenario: Non-Git child directories are skipped during tag deletion
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** local tag `v1.0.0` exists in `backend`
- **AND** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL delete local tag `v1.0.0` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: Partial failures do not stop other tag deletion attempts
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** local tag `v1.0.0` is missing in `backend`
- **AND** local tag `v1.0.0` exists in `frontend` and `tools`
- **AND** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL attempt tag deletion in all three child Git repositories
- **AND** the command SHALL delete local tag `v1.0.0` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed tag deletions are reported concisely
- **WHEN** tag deletion fails in `backend` with Git error `error: tag 'v1.0.0' not found.`
- **AND** tag deletion fails in `tools` with Git error `error: tag 'v1.0.0' not found.`
- **AND** user runs `git vmr tag -d v1.0.0`
- **THEN** stderr SHALL contain `error: tag 'v1.0.0' not found. (backend)`
- **AND** stderr SHALL contain `error: tag 'v1.0.0' not found. (tools)`
- **AND** each reported failure line SHALL use the first line of the underlying Git error followed by the repository name in parentheses

#### Scenario: Failed tag deletion reports are deterministic
- **WHEN** tag deletion fails in multiple child Git repositories
- **AND** tag deletion attempts are run in parallel
- **THEN** the reported failure lines SHALL be ordered deterministically by repository name

#### Scenario: Successful tag deletion output includes repository names
- **WHEN** local tag `v1.0.0` is deleted successfully in `backend`
- **AND** user runs `git vmr tag -d v1.0.0`
- **THEN** stdout SHALL contain Git's successful deletion message followed by ` (backend)`

## MODIFIED Requirements

### Requirement: Tag creation argument rules match supported Git tag behavior
The `git vmr tag` command SHALL accept at most one optional tag name positional argument for lightweight tag creation. It SHALL accept `-d` and `--delete` with exactly one tag name for local tag deletion. It SHALL reject unsupported annotated tag, signed tag, forced replacement, explicit target object, verification, filtering, formatting, pattern, and multi-tag deletion arguments during CLI argument validation.

#### Scenario: Tag listing remains unchanged
- **WHEN** user runs `git vmr tag` without a tag name or unsupported options
- **THEN** the command SHALL list local tags across child repositories

#### Scenario: Lightweight tag creation is supported
- **WHEN** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL create lightweight tag `v1.0.0` across child repositories

#### Scenario: Tag deletion is supported
- **WHEN** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL delete local tag `v1.0.0` across child repositories

#### Scenario: Delete flag requires tag name
- **WHEN** user runs `git vmr tag -d`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Multi-tag deletion is unsupported
- **WHEN** user runs `git vmr tag -d v1.0.0 v1.1.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Explicit target object is unsupported
- **WHEN** user runs `git vmr tag v1.0.0 HEAD~1`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Annotated tag creation is unsupported
- **WHEN** user runs `git vmr tag -a v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Forced tag replacement is unsupported
- **WHEN** user runs `git vmr tag -f v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Tag filtering remains unsupported
- **WHEN** user runs `git vmr tag --contains HEAD`
- **THEN** the command SHALL fail during CLI argument validation
