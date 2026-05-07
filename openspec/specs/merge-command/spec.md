# merge-command Specification

## Purpose
Define how `git vmr merge` applies a requested commit-ish across immediate child Git repositories in a VMR while preserving Git's per-repository merge behavior, reporting repository-specific results, and respecting existing VMR root discovery rules.

## Requirements
### Requirement: Merge command attempts merges across child repositories
The `git vmr merge <commit-ish>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to merge the requested commit-ish in every immediate child Git repository.

#### Scenario: Clean merge in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can merge `feature/auth`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL merge `feature/auth` in both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can merge `feature/auth`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL merge `feature/auth` in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL succeed with no output

### Requirement: Merge command delegates commit-ish resolution to Git
The `git vmr merge <commit-ish>` command SHALL pass the provided commit-ish to `git merge` in each child repository without pre-filtering repositories based on local branch existence.

#### Scenario: Missing ref fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has a mergeable `feature/auth` ref
- **AND** `frontend` does not have a `feature/auth` ref
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL attempt the merge in both repositories
- **AND** the command SHALL merge `feature/auth` in `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Detached commit can be merged
- **WHEN** a child repository can merge commit hash `abc1234`
- **AND** user runs `git vmr merge abc1234`
- **THEN** the command SHALL pass `abc1234` to Git as the merge target

### Requirement: Merge command is best-effort across repositories
The `git vmr merge <commit-ish>` command SHALL attempt a merge in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted merge fails.

#### Scenario: Failure does not stop later repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** merging `feature/auth` fails in `backend`
- **AND** merging `feature/auth` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL attempt merges in `backend`, `frontend`, and `tools`
- **AND** the command SHALL merge `feature/auth` in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful merges are not rolled back after later failure
- **WHEN** merging `feature/auth` succeeds in `backend`
- **AND** merging `feature/auth` fails in `frontend`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the successful merge in `backend` SHALL remain applied
- **AND** the command SHALL exit with a non-zero status

### Requirement: Merge conflicts are preserved for user resolution
When Git reports merge conflicts in a child repository, the `git vmr merge <commit-ish>` command SHALL treat that repository as failed and SHALL leave the repository in Git's normal conflicted merge state.

#### Scenario: Conflicted repository remains conflicted
- **WHEN** merging `feature/auth` in `backend` produces conflicts
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL exit with a non-zero status
- **AND** `backend` SHALL remain in an in-progress conflicted merge state
- **AND** stderr SHALL report the `backend` merge failure

#### Scenario: Conflict does not stop clean repositories
- **WHEN** merging `feature/auth` conflicts in `backend`
- **AND** merging `feature/auth` can succeed in `frontend`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** the command SHALL leave `backend` in conflicted merge state
- **AND** the command SHALL merge `feature/auth` in `frontend`
- **AND** the command SHALL exit with a non-zero status

### Requirement: Merge command reports repository-suffixed results
For each successful child repository merge, the `git vmr merge <commit-ish>` command SHALL write the first non-empty line of Git's merge stdout to stdout with the child repository name appended in parentheses. For each failed merge, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses.

#### Scenario: Successful merges print Git summary lines with repository suffixes
- **WHEN** merging `feature/auth` succeeds in `backend`
- **AND** Git prints `Updating abc1234..def5678`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** stdout SHALL contain `Updating abc1234..def5678 (backend)`

#### Scenario: Failed merges are reported with repository suffixes
- **WHEN** merging `feature/auth` fails in `backend` with Git error `merge: feature/auth - not something we can merge`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** stderr SHALL contain `merge: feature/auth - not something we can merge (backend)`

#### Scenario: Failed merge reports are deterministic
- **WHEN** merging fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr merge feature/auth`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Merge command uses existing VMR working directory behavior
The `git vmr merge <commit-ish>` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr merge feature/auth` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt merges across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend merge feature/auth`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt merges across immediate child Git repositories
