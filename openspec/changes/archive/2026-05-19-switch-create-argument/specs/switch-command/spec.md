## ADDED Requirements

### Requirement: Switch command creates and switches to new branches
The `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>` commands SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to create and switch to the requested branch in every immediate child Git repository.

#### Scenario: Create and switch in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL create local branch `feature/auth` in both `backend` and `frontend`
- **AND** the command SHALL switch both `backend` and `frontend` to `feature/auth`
- **AND** the command SHALL exit successfully

#### Scenario: Short create flag is accepted
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr switch -c feature/auth`
- **THEN** the command SHALL create local branch `feature/auth` in `backend`
- **AND** the command SHALL switch `backend` to `feature/auth`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped during create
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL create and switch to local branch `feature/auth` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found during create
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Switch create delegates repository semantics to Git
The `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>` commands SHALL pass the requested branch name to `git switch --create` in each child repository without pre-filtering repositories based on branch existence, start-point validity, worktree cleanliness, branch naming validity, hook behavior, or other Git-enforced conditions.

#### Scenario: Existing branch fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` already has local branch `feature/auth`
- **AND** `frontend` can create local branch `feature/auth`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL attempt create-and-switch in both repositories
- **AND** the command SHALL create and switch to `feature/auth` in `frontend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `backend` failure

#### Scenario: Dirty worktree protection during create is handled by Git
- **WHEN** creating and switching to `feature/auth` would be rejected by Git in child repository `backend` because of local worktree state
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL let Git reject the create-and-switch in `backend`
- **AND** `backend` SHALL remain on its original branch
- **AND** stderr SHALL report the `backend` failure

### Requirement: Switch create is best-effort across repositories
The `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>` commands SHALL attempt create-and-switch in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted create-and-switch fails.

#### Scenario: Create failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** creating `feature/auth` fails in `backend`
- **AND** creating and switching to `feature/auth` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** the command SHALL attempt create-and-switch in `backend`, `frontend`, and `tools`
- **AND** the command SHALL create and switch to `feature/auth` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful creates are not rolled back after failure
- **WHEN** creating and switching to `feature/auth` succeeds in `frontend`
- **AND** creating `feature/auth` fails in `backend`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** `frontend` SHALL remain on `feature/auth`
- **AND** the command SHALL exit with a non-zero status

### Requirement: Switch create deduplicates successful output and reports failures
For create-and-switch attempts, stdout SHALL contain each unique successful Git summary line once, without repository suffixes. For each failed child repository create-and-switch attempt, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses.

#### Scenario: Identical successful create output is printed once
- **WHEN** creating and switching to `feature/auth` succeeds in `backend` and `frontend`
- **AND** Git prints `Switched to a new branch 'feature/auth'` for both repositories
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** stdout SHALL contain `Switched to a new branch 'feature/auth'` exactly once
- **AND** stdout SHALL NOT contain `(backend)`
- **AND** stdout SHALL NOT contain `(frontend)`

#### Scenario: Mixed create success and failure reports deduplicated success and repository failure
- **WHEN** creating and switching to `feature/auth` succeeds in `frontend` and `tools`
- **AND** Git prints `Switched to a new branch 'feature/auth'` for both successful repositories
- **AND** creating `feature/auth` fails in `backend` with Git error `fatal: a branch named 'feature/auth' already exists`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** stdout SHALL contain `Switched to a new branch 'feature/auth'` exactly once
- **AND** stderr SHALL contain `fatal: a branch named 'feature/auth' already exists (backend)`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed create reports are deterministic
- **WHEN** creating `feature/auth` fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr switch --create feature/auth`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Switch create rejects start-point arguments
The `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>` commands SHALL accept exactly one branch name argument and SHALL reject additional start-point arguments.

#### Scenario: Long create flag rejects start point
- **WHEN** user runs `git vmr switch --create feature/auth main`
- **THEN** command parsing SHALL fail
- **AND** no child repository create-and-switch attempts SHALL be made

#### Scenario: Short create flag rejects start point
- **WHEN** user runs `git vmr switch -c feature/auth main`
- **THEN** command parsing SHALL fail
- **AND** no child repository create-and-switch attempts SHALL be made
