# branch-command Specification

## Purpose
Define `git vmr branch` behavior for listing local branches across immediate child repositories in a virtual monorepo, including branch-centric grouping, active markers, detached HEAD reporting, and existing VMR working directory handling.

## Requirements

### Requirement: Branch command lists local branches across child repositories
The `git vmr branch` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and list local branch refs from each immediate child Git repository.

#### Scenario: Multiple child repositories with local branches
- **WHEN** a VMR contains child Git repositories `backend` and `frontend` and both have local branch `main`
- **THEN** `git vmr branch` SHALL include a `main` branch line

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **THEN** `git vmr branch` SHALL list branches from `backend` and SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **THEN** `git vmr branch` SHALL succeed with no output

### Requirement: Branch output is branch-centric
The `git vmr branch` command SHALL group output by local branch name across discovered child Git repositories and SHALL render branch lines in deterministic branch-name order.

#### Scenario: Shared branch omits repository list
- **WHEN** discovered repositories `backend` and `frontend` both have local branch `main`
- **THEN** the output SHALL contain `main` without a repository list

#### Scenario: Partial branch lists repositories
- **WHEN** discovered repositories are `backend` and `frontend` and only `frontend` has local branch `feature/auth`
- **THEN** the output SHALL contain `feature/auth (frontend)`

#### Scenario: Partial branch lists multiple repositories
- **WHEN** discovered repositories are `backend`, `frontend`, and `tools` and only `backend` and `frontend` have local branch `release/1.2`
- **THEN** the output SHALL contain `release/1.2 (backend, frontend)`

### Requirement: Active branch lines use Git branch markers
Branch output lines SHALL use Git's branch marker convention: a line SHALL begin with `*` when that branch is checked out in at least one discovered repository, and SHALL begin with a space when that branch is not checked out in any discovered repository.

#### Scenario: Active shared branch
- **WHEN** local branch `main` exists in all discovered repositories and is checked out in at least one repository
- **THEN** the output SHALL contain `* main`

#### Scenario: Inactive shared branch
- **WHEN** local branch `release/1.2` exists in all discovered repositories and is not checked out in any repository
- **THEN** the output SHALL contain `  release/1.2`

#### Scenario: Active partial branch
- **WHEN** local branch `feature/auth` exists only in `frontend` and is checked out in `frontend`
- **THEN** the output SHALL contain `* feature/auth (frontend)`

### Requirement: Detached HEAD repositories are shown separately
Repositories in detached HEAD state SHALL render as individual active lines using the format `* (HEAD detached at <short-hash>) (<repo-name>)`.

#### Scenario: Detached repository
- **WHEN** `frontend` is in detached HEAD state at commit `9773cf0`
- **THEN** the output SHALL contain `* (HEAD detached at 9773cf0) (frontend)`

#### Scenario: Multiple detached repositories
- **WHEN** `backend` and `frontend` are both in detached HEAD state
- **THEN** the output SHALL contain one detached HEAD line for `backend` and one detached HEAD line for `frontend`

### Requirement: Branch command uses existing VMR working directory behavior
The `git vmr branch` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr branch` from inside child repository `frontend` and a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and list branches across all immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend branch` and `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and list branches across its immediate child Git repositories

### Requirement: Branch command fails on child repository query errors
If a child directory appears to be a Git repository but branch information cannot be queried from it, the `git vmr branch` command SHALL fail with a non-zero exit status and an error message.

#### Scenario: Corrupted child Git repository
- **WHEN** a VMR contains a child directory with an invalid `.git` entry
- **THEN** `git vmr branch` SHALL fail and report that branch information could not be read

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
- **THEN** stderr SHALL contain `fatal: error: the branch 'feature/auth' is not fully merged (backend)`
- **AND** stderr SHALL contain `fatal: error: branch 'feature/auth' not found (tools)`
- **AND** each reported failure line SHALL use the first line of the underlying Git error followed by the repository name in parentheses

#### Scenario: Failed branch deletion reports are deterministic
- **WHEN** branch deletion fails in multiple child Git repositories
- **AND** branch deletion attempts are run in parallel
- **THEN** the reported failure lines SHALL be ordered deterministically by repository name

#### Scenario: Successful branch deletion output includes repository names
- **WHEN** local branch `feature/auth` is deleted successfully in `backend`
- **AND** user runs `git vmr branch -d feature/auth`
- **THEN** stdout SHALL contain Git's successful deletion message followed by ` (backend)`

### Requirement: Branch deletion flags match Git branch argument rules
The `git vmr branch` command SHALL accept `-d` and `-D` as mutually exclusive branch deletion flags, and each deletion flag SHALL require exactly one branch name argument.

#### Scenario: Delete flags are mutually exclusive
- **WHEN** user runs `git vmr branch -d -D feature/auth`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Delete flag requires branch name
- **WHEN** user runs `git vmr branch -d`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Branch listing remains unchanged
- **WHEN** user runs `git vmr branch` without `-d`, `-D`, or a branch name
- **THEN** the command SHALL list local branches across child repositories

#### Scenario: Branch creation remains unchanged
- **WHEN** user runs `git vmr branch feature/auth` without `-d` or `-D`
- **THEN** the command SHALL create local branch `feature/auth` across child repositories
