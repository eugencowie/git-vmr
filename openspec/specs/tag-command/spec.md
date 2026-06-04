# tag-command Specification

## Purpose
Describe how `git vmr tag` lists local tags across immediate child Git repositories in a virtual monorepo, including tag-centric output and list-only command behavior.
## Requirements
### Requirement: Tag command lists local tags across child repositories
The `git vmr tag` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and list local tag refs from each immediate child Git repository.

#### Scenario: Multiple child repositories with local tags
- **WHEN** a VMR contains child Git repositories `backend` and `frontend` and both have local tag `v1.0.0`
- **THEN** `git vmr tag` SHALL include a `v1.0.0` tag line

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **THEN** `git vmr tag` SHALL list tags from `backend` and SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **THEN** `git vmr tag` SHALL succeed with no output

#### Scenario: No local tags found
- **WHEN** a VMR contains child Git repositories but none has local tags
- **THEN** `git vmr tag` SHALL succeed with no output

### Requirement: Tag output is tag-centric
The `git vmr tag` command SHALL group output by local tag name across discovered child Git repositories and SHALL render tag lines in deterministic tag-name order.

#### Scenario: Shared tag omits repository list
- **WHEN** discovered repositories `backend` and `frontend` both have local tag `v1.0.0`
- **THEN** the output SHALL contain `v1.0.0` without a repository list

#### Scenario: Partial tag lists repositories
- **WHEN** discovered repositories are `backend` and `frontend` and only `frontend` has local tag `v1.1.0`
- **THEN** the output SHALL contain `v1.1.0 (frontend)`

#### Scenario: Partial tag lists multiple repositories
- **WHEN** discovered repositories are `backend`, `frontend`, and `tools` and only `backend` and `frontend` have local tag `v1.2.0`
- **THEN** the output SHALL contain `v1.2.0 (backend, frontend)`

### Requirement: Tag command uses existing VMR working directory behavior
The `git vmr tag` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr tag` from inside child repository `frontend` and a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and list tags across all immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend tag` and `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and list tags across its immediate child Git repositories

### Requirement: Tag command fails on child repository query errors
If a child directory appears to be a Git repository but tag information cannot be queried from it, the `git vmr tag` command SHALL fail with a non-zero exit status and an error message.

#### Scenario: Corrupted child Git repository
- **WHEN** a VMR contains a child directory with an invalid `.git` entry
- **THEN** `git vmr tag` SHALL fail and report that tag information could not be read

### Requirement: Tag command creates lightweight tags across child repositories
When a tag name is provided, the `git vmr tag <tag-name>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to create the named lightweight tag at `HEAD` in every immediate child Git repository. Tag creation attempts SHALL be independent: a failure in one child repository SHALL NOT prevent tag creation from being attempted in other discovered child Git repositories.

#### Scenario: Create tag in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL create lightweight tag `v1.0.0` in both `backend` and `frontend`
- **AND** the command SHALL succeed with no output

#### Scenario: Non-Git child directories are skipped during tag creation
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL create lightweight tag `v1.0.0` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: Partial failures do not stop other tag creation attempts
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** `v1.0.0` already exists in `backend`
- **AND** `v1.0.0` can be created in `frontend` and `tools`
- **AND** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL attempt tag creation in all three child Git repositories
- **AND** the command SHALL create lightweight tag `v1.0.0` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed tag creations are reported concisely
- **WHEN** tag creation fails in `backend` with Git error `fatal: tag 'v1.0.0' already exists`
- **AND** tag creation fails in `tools` with Git error `fatal: Failed to resolve 'HEAD' as a valid ref.`
- **AND** user runs `git vmr tag v1.0.0`
- **THEN** stderr SHALL contain `fatal: tag 'v1.0.0' already exists (backend)`
- **AND** stderr SHALL contain `fatal: Failed to resolve 'HEAD' as a valid ref. (tools)`
- **AND** each reported failure line SHALL use the first line of the underlying Git error followed by the repository name in parentheses

#### Scenario: Failed tag creation reports are deterministic
- **WHEN** tag creation fails in multiple child Git repositories
- **AND** tag creation attempts are run in parallel
- **THEN** the reported failure lines SHALL be ordered deterministically by repository name

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

