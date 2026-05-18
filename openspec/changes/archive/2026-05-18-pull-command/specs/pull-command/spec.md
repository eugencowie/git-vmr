# pull-command Specification

## Purpose
Define `git vmr pull [<repository> [<refspec>...]]` behavior for pulling across immediate child repositories in a virtual monorepo, including Git-delegated argument handling, best-effort execution, deterministic reporting, and existing VMR working directory handling.

## ADDED Requirements

### Requirement: Pull command attempts pulls across child repositories
The `git vmr pull [<repository> [<refspec>...]]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt a pull in every immediate child Git repository.

#### Scenario: Pull in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can pull from their configured default upstreams
- **AND** user runs `git vmr pull`
- **THEN** the command SHALL pull in both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can pull from its configured default upstream
- **AND** user runs `git vmr pull`
- **THEN** the command SHALL pull in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr pull`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Pull command supports repository and refspec arguments
The `git vmr pull [<repository> [<refspec>...]]` command SHALL pass the optional repository argument and any following refspec arguments to `git pull` in each child repository in the same order provided by the user.

#### Scenario: Pull with repository argument
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have a remote named `origin`
- **AND** user runs `git vmr pull origin`
- **THEN** the command SHALL run `git pull origin` in both `backend` and `frontend`

#### Scenario: Pull with repository and single refspec
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can pull refspec `main` from `origin`
- **AND** user runs `git vmr pull origin main`
- **THEN** the command SHALL run `git pull origin main` in both `backend` and `frontend`

#### Scenario: Pull with repository and multiple refspecs
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can pull refspecs `main` and `release` from `origin`
- **AND** user runs `git vmr pull origin main release`
- **THEN** the command SHALL run `git pull origin main release` in both `backend` and `frontend`

#### Scenario: Single positional argument is treated as repository
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr pull main`
- **THEN** the command SHALL run `git pull main` in `backend`
- **AND** the command SHALL NOT reinterpret `main` as a refspec for the default remote

### Requirement: Pull command delegates Git argument and repository-state resolution to child repositories
The `git vmr pull [<repository> [<refspec>...]]` command SHALL pass pull arguments to Git without pre-filtering repositories based on remote existence, upstream configuration, branch state, working tree cleanliness, repository URL reachability, pull strategy configuration, or refspec validity.

#### Scenario: Missing remote fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has a remote named `origin`
- **AND** `frontend` does not have a remote named `origin`
- **AND** user runs `git vmr pull origin`
- **THEN** the command SHALL attempt the pull in both repositories
- **AND** the command SHALL pull in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Repository URL is passed through
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr pull /repos/project.git main`
- **THEN** the command SHALL pass `/repos/project.git` to Git as the pull repository argument
- **AND** the command SHALL pass `main` to Git as a refspec argument

#### Scenario: Missing upstream is delegated to Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** `backend` has no upstream configured for the current branch
- **AND** user runs `git vmr pull`
- **THEN** the command SHALL attempt `git pull` in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `backend` failure

### Requirement: Pull command is best-effort across repositories
The `git vmr pull [<repository> [<refspec>...]]` command SHALL attempt a pull in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted pull fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** pulling from `origin` fails in `backend`
- **AND** pulling from `origin` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr pull origin`
- **THEN** the command SHALL attempt pulls in `backend`, `frontend`, and `tools`
- **AND** the command SHALL pull in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful pulls are not rolled back after failure
- **WHEN** pulling from `origin` succeeds in `backend`
- **AND** pulling from `origin` fails in `frontend`
- **AND** user runs `git vmr pull origin`
- **THEN** the successful pull in `backend` SHALL remain applied
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Conflicted pull remains for user resolution
- **WHEN** pulling in `backend` creates a merge conflict
- **AND** pulling in `frontend` succeeds
- **AND** user runs `git vmr pull`
- **THEN** the command SHALL report the `backend` failure
- **AND** the successful pull in `frontend` SHALL remain applied
- **AND** the conflict state in `backend` SHALL remain for user resolution
- **AND** the command SHALL exit with a non-zero status

### Requirement: Pull command runs child repository attempts in parallel
The `git vmr pull [<repository> [<refspec>...]]` command SHALL run child repository pull attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** pulling fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr pull origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Pull command reports repository-suffixed results
For each successful child repository pull that emits output, stdout SHALL contain the first non-empty line from Git stdout, or Git stderr if stdout is empty, with the child repository name appended. Successful child repository pulls that emit no output SHALL NOT produce aggregate output. For each failed pull, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended.

#### Scenario: Successful pull prints Git summary line with repository suffix
- **WHEN** pulling in `backend` succeeds and Git emits `Updating abc123..def456` on stdout
- **AND** user runs `git vmr pull`
- **THEN** stdout SHALL contain `Updating abc123..def456 (backend)`

#### Scenario: Already up-to-date pull prints repository suffix
- **WHEN** pulling in `backend` succeeds and Git emits `Already up to date.` on stdout
- **AND** user runs `git vmr pull`
- **THEN** stdout SHALL contain `Already up to date. (backend)`

#### Scenario: Successful pull with no Git output is quiet
- **WHEN** pulling in every discovered child Git repository succeeds with no Git stdout or stderr output
- **AND** user runs `git vmr pull`
- **THEN** stdout and stderr SHALL be empty

#### Scenario: Failed pull reports repository suffix
- **WHEN** pulling from `origin` fails in `backend` with Git error `fatal: 'origin' does not appear to be a git repository`
- **AND** user runs `git vmr pull origin`
- **THEN** stderr SHALL contain `fatal: 'origin' does not appear to be a git repository (backend)`

#### Scenario: Failed pull reports are deterministic
- **WHEN** pulling fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr pull origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Pull command uses existing VMR working directory behavior
The `git vmr pull [<repository> [<refspec>...]]` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr pull origin` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt pulls across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend pull origin`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt pulls across its immediate child Git repositories
