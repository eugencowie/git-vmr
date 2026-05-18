# push-command Specification

## Purpose
Define `git vmr push [<repository> [<refspec>...]]` behavior for pushing across immediate child repositories in a virtual monorepo, including Git-delegated argument handling, best-effort execution, deterministic reporting, and existing VMR working directory handling.

## ADDED Requirements

### Requirement: Push command attempts pushes across child repositories
The `git vmr push [<repository> [<refspec>...]]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt a push in every immediate child Git repository.

#### Scenario: Push in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can push to their configured default push destinations
- **AND** user runs `git vmr push`
- **THEN** the command SHALL push in both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can push to its configured default push destination
- **AND** user runs `git vmr push`
- **THEN** the command SHALL push in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr push`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Push command supports repository and refspec arguments
The `git vmr push [<repository> [<refspec>...]]` command SHALL pass the optional repository argument and any following refspec arguments to `git push` in each child repository in the same order provided by the user.

#### Scenario: Push with repository argument
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have a remote named `origin`
- **AND** user runs `git vmr push origin`
- **THEN** the command SHALL run `git push origin` in both `backend` and `frontend`

#### Scenario: Push with repository and single refspec
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can push refspec `main` to `origin`
- **AND** user runs `git vmr push origin main`
- **THEN** the command SHALL run `git push origin main` in both `backend` and `frontend`

#### Scenario: Push with repository and multiple refspecs
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can push refspecs `main` and `release` to `origin`
- **AND** user runs `git vmr push origin main release`
- **THEN** the command SHALL run `git push origin main release` in both `backend` and `frontend`

#### Scenario: Single positional argument is treated as repository
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr push main`
- **THEN** the command SHALL run `git push main` in `backend`
- **AND** the command SHALL NOT reinterpret `main` as a refspec for the default remote

### Requirement: Push command delegates Git argument and repository-state resolution to child repositories
The `git vmr push [<repository> [<refspec>...]]` command SHALL pass push arguments to Git without pre-filtering repositories based on remote existence, upstream configuration, default push remote, branch state, working tree cleanliness, repository URL reachability, authentication, remote hook behavior, push policy, or refspec validity.

#### Scenario: Missing remote fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has a remote named `origin`
- **AND** `frontend` does not have a remote named `origin`
- **AND** user runs `git vmr push origin`
- **THEN** the command SHALL attempt the push in both repositories
- **AND** the command SHALL push in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Repository URL is passed through
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr push /repos/project.git main`
- **THEN** the command SHALL pass `/repos/project.git` to Git as the push repository argument
- **AND** the command SHALL pass `main` to Git as a refspec argument

#### Scenario: Missing upstream is delegated to Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** `backend` has no upstream or default push destination configured for the current branch
- **AND** user runs `git vmr push`
- **THEN** the command SHALL attempt `git push` in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `backend` failure

#### Scenario: Push rejection is delegated to Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** Git rejects `backend` because the remote contains work not present locally
- **AND** user runs `git vmr push`
- **THEN** the command SHALL report the `backend` failure
- **AND** the command SHALL exit with a non-zero status

### Requirement: Push command is best-effort across repositories
The `git vmr push [<repository> [<refspec>...]]` command SHALL attempt a push in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted push fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** pushing to `origin` fails in `backend`
- **AND** pushing to `origin` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr push origin`
- **THEN** the command SHALL attempt pushes in `backend`, `frontend`, and `tools`
- **AND** the command SHALL push in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful pushes are not rolled back after failure
- **WHEN** pushing to `origin` succeeds in `backend`
- **AND** pushing to `origin` fails in `frontend`
- **AND** user runs `git vmr push origin`
- **THEN** the successful push in `backend` SHALL remain published
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Rejected push remains for user resolution
- **WHEN** pushing in `backend` is rejected by Git
- **AND** pushing in `frontend` succeeds
- **AND** user runs `git vmr push`
- **THEN** the command SHALL report the `backend` failure
- **AND** the successful push in `frontend` SHALL remain published
- **AND** the rejected state in `backend` SHALL remain for user resolution
- **AND** the command SHALL exit with a non-zero status

### Requirement: Push command runs child repository attempts in parallel
The `git vmr push [<repository> [<refspec>...]]` command SHALL run child repository push attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** pushing fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr push origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Push command reports repository-suffixed results
For each successful child repository push that emits output, stdout SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended. Successful child repository pushes that emit no output SHALL NOT produce aggregate output. For each failed push, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended.

#### Scenario: Successful push prints Git destination line with repository suffix
- **WHEN** pushing in `backend` succeeds and Git emits `To /repos/backend.git` on stderr
- **AND** user runs `git vmr push`
- **THEN** stdout SHALL contain `To /repos/backend.git (backend)`

#### Scenario: Everything up-to-date push prints repository suffix
- **WHEN** pushing in `backend` succeeds and Git emits `Everything up-to-date` on stderr
- **AND** user runs `git vmr push`
- **THEN** stdout SHALL contain `Everything up-to-date (backend)`

#### Scenario: Successful push with no Git output is quiet
- **WHEN** pushing in every discovered child Git repository succeeds with no Git stdout or stderr output
- **AND** user runs `git vmr push`
- **THEN** stdout and stderr SHALL be empty

#### Scenario: Failed push reports repository suffix
- **WHEN** pushing to `origin` fails in `backend` with Git error `fatal: 'origin' does not appear to be a git repository`
- **AND** user runs `git vmr push origin`
- **THEN** stderr SHALL contain `fatal: 'origin' does not appear to be a git repository (backend)`

#### Scenario: Failed push reports are deterministic
- **WHEN** pushing fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr push origin`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Push command uses existing VMR working directory behavior
The `git vmr push [<repository> [<refspec>...]]` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr push origin` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt pushes across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend push origin`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt pushes across its immediate child Git repositories
