# switch-command Specification

## Purpose
Define `git vmr switch <branch-name>` behavior for switching branches across immediate child repositories in a virtual monorepo, including best-effort execution, Git-delegated branch resolution, deterministic reporting, and existing VMR working directory handling.

## Requirements

### Requirement: Switch command attempts branch switches across child repositories
The `git vmr switch <branch-name>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to switch every immediate child Git repository to the requested branch.

#### Scenario: Switch branch in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have local branch `feature/auth`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL switch both `backend` and `frontend` to `feature/auth`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` has local branch `feature/auth`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL switch `backend` to `feature/auth`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Switch command delegates branch resolution to Git
The `git vmr switch <branch-name>` command SHALL pass the provided branch name to `git switch` in each child repository without pre-filtering repositories based on local branch existence.

#### Scenario: Missing branch fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has local branch `feature/auth`
- **AND** `frontend` does not have local branch `feature/auth`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL attempt the switch in both repositories
- **AND** the command SHALL switch `backend` to `feature/auth`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Dirty worktree protection is handled by Git
- **WHEN** switching to `feature/auth` would overwrite local changes in child repository `backend`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL let Git reject the switch in `backend`
- **AND** `backend` SHALL remain on its original branch
- **AND** stderr SHALL report the `backend` failure

### Requirement: Switch command is best-effort across repositories
The `git vmr switch <branch-name>` command SHALL attempt a switch in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted switch fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** switching to `feature/auth` fails in `backend`
- **AND** switching to `feature/auth` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** the command SHALL attempt switches in `backend`, `frontend`, and `tools`
- **AND** the command SHALL switch `frontend` and `tools` to `feature/auth`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful switches are not rolled back after failure
- **WHEN** switching to `feature/auth` succeeds in `backend`
- **AND** switching to `feature/auth` fails in `frontend`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** `backend` SHALL remain on `feature/auth`
- **AND** the command SHALL exit with a non-zero status

### Requirement: Switch command runs child repository attempts in parallel
The `git vmr switch <branch-name>` command SHALL run child repository switch attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** switching fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Switch command reports repository-suffixed results
For each successful child repository switch, stdout SHALL contain the first non-empty line from Git stdout or Git stderr with the child repository name appended in parentheses. For each failed child repository switch, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses.

#### Scenario: Successful switches print Git summary lines with repository suffixes
- **WHEN** switching to `feature/auth` succeeds in `backend`
- **AND** Git prints `Switched to branch 'feature/auth'`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stdout SHALL contain `Switched to branch 'feature/auth' (backend)`

#### Scenario: Failed switches are reported with repository suffixes
- **WHEN** switching to `feature/auth` fails in `backend` with Git error `invalid reference: feature/auth`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stderr SHALL contain `invalid reference: feature/auth (backend)`

#### Scenario: Failed switch reports are deterministic
- **WHEN** switching fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Switch command uses existing VMR working directory behavior
The `git vmr switch <branch-name>` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr switch feature/auth` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt switches across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend switch feature/auth`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt switches across immediate child Git repositories
