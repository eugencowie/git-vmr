## MODIFIED Requirements

### Requirement: Branch command deletes local branches across child repositories
When `-d` or `-D` is provided with a branch name, the `git vmr branch` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to delete the named local branch in every immediate child Git repository. Branch deletion attempts SHALL be independent: a failure in one child repository SHALL NOT prevent branch deletion from being attempted in other discovered child Git repositories.

#### Scenario: Safely delete branch in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** local branch `feature/auth` exists in both repositories and is eligible for `git branch -d`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** the command SHALL delete local branch `feature/auth` in both repositories using Git's safe deletion semantics
- **AND** the command SHALL succeed

#### Scenario: Force delete branch in every child repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** local branch `feature/auth` exists in both repositories
- **AND** user runs `git vmr branch -D feature/auth`
- **THEN** the command SHALL delete local branch `feature/auth` in both repositories using Git's force deletion semantics
- **AND** the command SHALL succeed

#### Scenario: Non-Git child directories are skipped during branch deletion
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** local branch `feature/auth` exists in `backend`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** the command SHALL delete local branch `feature/auth` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: Partial failures do not stop other branch deletion attempts
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** local branch `feature/auth` cannot be safely deleted in `backend`
- **AND** local branch `feature/auth` can be safely deleted in `frontend` and `tools`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** the command SHALL attempt branch deletion in all three child Git repositories
- **AND** the command SHALL delete local branch `feature/auth` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed branch deletions are reported concisely
- **WHEN** branch deletion fails in `backend` with Git error `error: the branch 'feature/auth' is not fully merged`
- **AND** branch deletion fails in `tools` with Git error `error: branch 'feature/auth' not found`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** stderr SHALL contain `error: the branch 'feature/auth' is not fully merged (backend)`
- **AND** stderr SHALL contain `error: branch 'feature/auth' not found (tools)`
- **AND** each reported failure line SHALL use the first line of the underlying Git error followed by the repository name in parentheses

#### Scenario: Failed branch deletion reports are deterministic
- **WHEN** branch deletion fails in multiple child Git repositories
- **AND** branch deletion attempts are run in parallel
- **THEN** the reported failure lines SHALL be ordered deterministically by repository name

#### Scenario: Successful branch deletion output includes repository names
- **WHEN** local branch `feature/auth` is deleted successfully in `backend`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** stdout SHALL contain Git's successful deletion message followed by ` (backend)`
