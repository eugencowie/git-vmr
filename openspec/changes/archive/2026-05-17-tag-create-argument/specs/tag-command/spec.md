## ADDED Requirements

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
The `git vmr tag` command SHALL accept at most one optional tag name positional argument. It SHALL reject unsupported annotated tag, signed tag, forced replacement, explicit target object, deletion, verification, filtering, formatting, and pattern arguments during CLI argument validation.

#### Scenario: Tag listing remains unchanged
- **WHEN** user runs `git vmr tag` without a tag name or unsupported options
- **THEN** the command SHALL list local tags across child repositories

#### Scenario: Lightweight tag creation is supported
- **WHEN** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL create lightweight tag `v1.0.0` across child repositories

#### Scenario: Explicit target object is unsupported
- **WHEN** user runs `git vmr tag v1.0.0 HEAD~1`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Annotated tag creation is unsupported
- **WHEN** user runs `git vmr tag -a v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Forced tag replacement is unsupported
- **WHEN** user runs `git vmr tag -f v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Tag deletion remains unsupported
- **WHEN** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Tag filtering remains unsupported
- **WHEN** user runs `git vmr tag --contains HEAD`
- **THEN** the command SHALL fail during CLI argument validation

## REMOVED Requirements

### Requirement: Tag command is list-only
**Reason**: `git vmr tag` now supports the plain Git lightweight creation form `git vmr tag <tag-name>` in addition to no-argument list mode.

**Migration**: Use `git vmr tag` for existing list behavior. Use `git vmr tag <tag-name>` to create a lightweight tag at `HEAD` in each child repository. Unsupported tag flags and extra operands continue to fail during CLI argument validation.
