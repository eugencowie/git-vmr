# push-command Specification

## Purpose
Define `git vmr push [<repository> [<refspec>...]]` behavior for pushing across immediate child repositories in a virtual monorepo, including Git-delegated argument handling, best-effort execution, deterministic reporting, and existing VMR working directory handling.

## Requirements

### Requirement: Push command attempts pushes across child repositories
The `git vmr push [<repository> [<refspec>...]]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt a push in every immediate child Git repository whose push classifies as useful.

#### Scenario: Push in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have useful pushes and can push to their configured default push destinations
- **AND** user runs `git vmr push`
- **THEN** the command SHALL push in both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` has a useful push and can push to its configured default push destination
- **AND** user runs `git vmr push`
- **THEN** the command SHALL push in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr push`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Push command supports repository and refspec arguments
The `git vmr push [<repository> [<refspec>...]]` command SHALL pass the optional repository argument and the useful subset of any following refspec arguments to `git push` in each child repository, preserving the order provided by the user.

#### Scenario: Push with repository argument
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have a remote named `origin` and useful pushes
- **AND** user runs `git vmr push origin`
- **THEN** the command SHALL run `git push origin` in both `backend` and `frontend`

#### Scenario: Push with repository and single refspec
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** refspec `main` classifies as useful in both repositories
- **AND** user runs `git vmr push origin main`
- **THEN** the command SHALL run `git push origin main` in both `backend` and `frontend`

#### Scenario: Push with repository and multiple refspecs
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** refspecs `main` and `release` classify as useful in both repositories
- **AND** user runs `git vmr push origin main release`
- **THEN** the command SHALL run `git push origin main release` in both `backend` and `frontend`

#### Scenario: Single positional argument is treated as repository
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr push main`
- **THEN** the command SHALL treat `main` as the repository argument
- **AND** the command SHALL NOT reinterpret `main` as a refspec for the default remote

### Requirement: Push command attempts only useful pushes
The `git vmr push [<repository> [<refspec>...]]` command SHALL attempt only useful pushes: pushes proven, from the local remote-tracking refs of the target remote alone, to transmit novel commits or to update or delete a ref that already exists on the target remote. The command SHALL skip, with a reported reason, every push it cannot prove useful. The command SHALL NOT contact the network to gather evidence.

#### Scenario: Feature branch with no new commits is skipped
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `docs`
- **AND** all three are on branch `feature` with no upstream configured
- **AND** only `backend` and `frontend` have commits on `feature` not reachable from their remote-tracking refs
- **AND** user runs `git vmr push`
- **THEN** the command SHALL push in `backend` and `frontend`
- **AND** the command SHALL NOT run `git push` in `docs`
- **AND** stdout SHALL report the `docs` skip with its reason

#### Scenario: Update to an existing remote branch is delegated
- **WHEN** child repository `backend` is on branch `main`
- **AND** `refs/remotes/origin/main` exists in `backend`
- **AND** user runs `git vmr push`
- **THEN** the command SHALL run `git push` in `backend` regardless of whether `main` has novel commits

#### Scenario: Deleted-on-remote branch is not re-created
- **WHEN** child repository `backend` is on branch `feature` with upstream configuration pointing at `origin/feature`
- **AND** no remote-tracking ref for the push destination exists
- **AND** the tip of `feature` is reachable from `origin`'s remote-tracking refs
- **AND** user runs `git vmr push`
- **THEN** the command SHALL NOT run `git push` in `backend`
- **AND** stdout SHALL report the skip with its reason

#### Scenario: Stale local picture errs toward pushing
- **WHEN** child repository `backend` is on branch `feature` whose tip is not reachable from any remote-tracking ref of the target remote
- **AND** the remote already contains those commits because the local picture is stale
- **AND** user runs `git vmr push`
- **THEN** the command SHALL run `git push` in `backend`
- **AND** the command SHALL NOT contact the network before deciding to push

### Requirement: Push command filters refspecs individually
The `git vmr push <repository> <refspec>...` command SHALL classify each refspec per child repository and SHALL pass to `git push` only the useful subset, preserving the user's order. A child repository whose refspecs are all skippable SHALL receive no git invocation. There SHALL be no flag that disables the filter.

#### Scenario: Mixed refspec list pushes only the useful subset
- **WHEN** child repository `backend` has novel commits on `main` but branch `feature` would be an empty branch creation
- **AND** user runs `git vmr push origin main feature`
- **THEN** the command SHALL run `git push origin main` in `backend`
- **AND** stdout SHALL report the `feature` skip for `backend` with its reason

#### Scenario: Repository with no useful refspecs gets no invocation
- **WHEN** every refspec of `git vmr push origin feature` classifies as an empty branch creation in child repository `docs`
- **THEN** the command SHALL NOT run `git push` in `docs`
- **AND** stdout SHALL report the skip

### Requirement: Push command classifies refspecs by ref class
The usefulness test SHALL apply to branch refspecs only. Delete refspecs SHALL always be pushed. Tag refspecs SHALL always be pushed without any usefulness check. Wildcard refspecs SHALL be skipped with a reason as unclassifiable. When the repository argument is a URL rather than a configured remote, the command SHALL pass the entire push through unfiltered.

#### Scenario: Tag refspec is always pushed
- **WHEN** child repository `backend` has local tag `v1.0.0` pointing at a commit already reachable from `origin`'s remote-tracking refs
- **AND** user runs `git vmr push origin v1.0.0`
- **THEN** the command SHALL run `git push origin v1.0.0` in `backend`

#### Scenario: Delete refspec is always pushed
- **WHEN** user runs `git vmr push origin :gone-branch`
- **THEN** the command SHALL run `git push origin :gone-branch` in every child Git repository

#### Scenario: Wildcard refspec is skipped
- **WHEN** user runs `git vmr push origin refs/heads/*:refs/heads/*`
- **THEN** the command SHALL NOT run `git push` in any child repository
- **AND** stdout SHALL report the skip with a reason identifying the refspec as unclassifiable

#### Scenario: URL repository argument delegates the whole push
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr push /repos/project.git main`
- **THEN** the command SHALL run `git push /repos/project.git main` in `backend` without any usefulness check

### Requirement: Push command reports skips without failing
Each skipped push SHALL be reported on stdout with its reason and repository attribution, aggregated by identical reason under the existing aggregate output rules. Skips SHALL NOT affect the exit code. A run in which every repository is skipped SHALL exit successfully and SHALL NOT be silent.

#### Scenario: All-skipped run exits zero and reports
- **WHEN** every child repository's push classifies as an empty branch creation
- **AND** user runs `git vmr push`
- **THEN** the command SHALL exit successfully
- **AND** stdout SHALL contain at least one skip line naming the skipped repositories

#### Scenario: Identical skip reasons are grouped
- **WHEN** pushes in `frontend`, `tools`, and `docs` are skipped for the same reason
- **AND** user runs `git vmr push`
- **THEN** stdout SHALL contain that reason exactly once with all three repositories attributed

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
