## ADDED Requirements

### Requirement: Fetch command attempts fetches across child repositories
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt a fetch in every immediate child Git repository.

#### Scenario: Fetch in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can fetch from their configured default remotes
- **AND** user runs `git vmr fetch`
- **THEN** the command SHALL fetch in both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can fetch from its configured default remote
- **AND** user runs `git vmr fetch`
- **THEN** the command SHALL fetch in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr fetch`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Fetch command supports repository and refspec arguments
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL pass the optional repository argument and any following refspec arguments to `git fetch` in each child repository in the same order provided by the user.

#### Scenario: Fetch with repository argument
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have a remote named `origin`
- **AND** user runs `git vmr fetch origin`
- **THEN** the command SHALL run `git fetch origin` in both `backend` and `frontend`

#### Scenario: Fetch with repository and single refspec
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can fetch refspec `main` from `origin`
- **AND** user runs `git vmr fetch origin main`
- **THEN** the command SHALL run `git fetch origin main` in both `backend` and `frontend`

#### Scenario: Fetch with repository and multiple refspecs
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can fetch refspecs `main` and `release:release` from `origin`
- **AND** user runs `git vmr fetch origin main release:release`
- **THEN** the command SHALL run `git fetch origin main release:release` in both `backend` and `frontend`

#### Scenario: Single positional argument is treated as repository
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr fetch main`
- **THEN** the command SHALL run `git fetch main` in `backend`
- **AND** the command SHALL NOT reinterpret `main` as a refspec for the default remote

### Requirement: Fetch command delegates Git argument resolution to child repositories
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL pass fetch arguments to Git without pre-filtering repositories based on remote existence, repository URL reachability, or refspec validity.

#### Scenario: Missing remote fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has a remote named `origin`
- **AND** `frontend` does not have a remote named `origin`
- **AND** user runs `git vmr fetch origin`
- **THEN** the command SHALL attempt the fetch in both repositories
- **AND** the command SHALL fetch in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Repository URL is passed through
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr fetch /repos/project.git main`
- **THEN** the command SHALL pass `/repos/project.git` to Git as the fetch repository argument
- **AND** the command SHALL pass `main` to Git as a refspec argument

### Requirement: Fetch command is best-effort across repositories
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL attempt a fetch in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted fetch fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** fetching from `origin` fails in `backend`
- **AND** fetching from `origin` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr fetch origin`
- **THEN** the command SHALL attempt fetches in `backend`, `frontend`, and `tools`
- **AND** the command SHALL fetch in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful fetches are not rolled back after failure
- **WHEN** fetching from `origin` succeeds in `backend`
- **AND** fetching from `origin` fails in `frontend`
- **AND** user runs `git vmr fetch origin`
- **THEN** the successful fetch in `backend` SHALL remain applied
- **AND** the command SHALL exit with a non-zero status

### Requirement: Fetch command runs child repository attempts in parallel
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL run child repository fetch attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** fetching fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr fetch origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Fetch command reports repository-suffixed results
For each successful child repository fetch that emits output, stdout SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended. Successful child repository fetches that emit no output SHALL NOT produce aggregate output. For each failed fetch, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended.

#### Scenario: Successful fetch prints Git summary line with repository suffix
- **WHEN** fetching in `backend` succeeds and Git emits `From /repos/backend` on stderr
- **AND** user runs `git vmr fetch`
- **THEN** stdout SHALL contain `From /repos/backend (backend)`

#### Scenario: Successful fetch with no Git output is quiet
- **WHEN** fetching in every discovered child Git repository succeeds with no Git stdout or stderr output
- **AND** user runs `git vmr fetch`
- **THEN** stdout and stderr SHALL be empty

#### Scenario: Failed fetch reports repository suffix
- **WHEN** fetching from `origin` fails in `backend` with Git error `fatal: 'origin' does not appear to be a git repository`
- **AND** user runs `git vmr fetch origin`
- **THEN** stderr SHALL contain `fatal: 'origin' does not appear to be a git repository (backend)`

#### Scenario: Failed fetch reports are deterministic
- **WHEN** fetching fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr fetch origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Fetch command uses existing VMR working directory behavior
The `git vmr fetch [<repository> [<refspec>...]]` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr fetch origin` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt fetches across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend fetch origin`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt fetches across its immediate child Git repositories
