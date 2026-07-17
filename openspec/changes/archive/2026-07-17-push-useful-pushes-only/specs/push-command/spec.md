## ADDED Requirements

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

## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Push command delegates Git argument and repository-state resolution to child repositories
**Reason**: Reversed by ADR 0007 — verbatim delegation creates empty branches on remotes (and fires CI) in every repo with nothing new during the core cross-VMR feature-branch workflow. Push now pre-filters on branch state by design.
**Migration**: Users who want an unfiltered push run `git vmr foreach 'git push ...'`. Git-side resolution of remotes, upstreams, authentication, hooks, and rejections remains delegated for every push that is attempted; only provably useless branch pushes are withheld. The "Repository URL is passed through" behavior survives under the refspec-classification requirement; delegated-failure scenarios (missing remote, missing upstream, push rejection) continue to hold for attempted pushes.
