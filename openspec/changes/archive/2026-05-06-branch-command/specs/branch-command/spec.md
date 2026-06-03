## ADDED Requirements

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
