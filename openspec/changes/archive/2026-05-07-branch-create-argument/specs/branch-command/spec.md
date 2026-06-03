## ADDED Requirements

### Requirement: Branch command creates local branches across child repositories
When a branch name is provided, the `git vmr branch <branch-name>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to create the named local branch in every immediate child Git repository. Branch creation attempts SHALL be independent: a failure in one child repository SHALL NOT prevent branch creation from being attempted in other discovered child Git repositories.

#### Scenario: Create branch in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr branch feature/auth`
- **THEN** the command SHALL create local branch `feature/auth` in both `backend` and `frontend`
- **AND** the command SHALL succeed with no output

#### Scenario: Non-Git child directories are skipped during branch creation
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr branch feature/auth`
- **THEN** the command SHALL create local branch `feature/auth` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: Partial failures do not stop other branch creation attempts
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** `feature/auth` already exists in `backend`
- **AND** `feature/auth` can be created in `frontend` and `tools`
- **AND** user runs `git vmr branch feature/auth`
- **THEN** the command SHALL attempt branch creation in all three child Git repositories
- **AND** the command SHALL create local branch `feature/auth` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed branch creations are reported concisely
- **WHEN** branch creation fails in `backend` with Git error `fatal: a branch named 'feature/auth' already exists`
- **AND** branch creation fails in `tools` with Git error `fatal: not a valid object name: 'HEAD'`
- **AND** user runs `git vmr branch feature/auth`
- **THEN** stderr SHALL contain `fatal: a branch named 'feature/auth' already exists (backend)`
- **AND** stderr SHALL contain `fatal: not a valid object name: 'HEAD' (tools)`
- **AND** each reported failure line SHALL use the first line of the underlying Git error followed by the repository name in parentheses

#### Scenario: Failed branch creation reports are deterministic
- **WHEN** branch creation fails in multiple child Git repositories
- **AND** branch creation attempts are run in parallel
- **THEN** the reported failure lines SHALL be ordered deterministically by repository name
