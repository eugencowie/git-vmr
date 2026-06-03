# commit-command Specification

## Purpose
Define how `git vmr commit` creates commits for staged changes across child repositories in a VMR while preserving existing working-directory behavior and reporting per-repository results clearly.

## Requirements
### Requirement: Commit command commits staged changes across child repositories
The `git vmr commit -m <message>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and create commits in child Git repositories that have staged changes.

#### Scenario: Commit staged changes in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories have staged changes
- **AND** user runs `git vmr commit -m "Implement new feature"`
- **THEN** the command SHALL create one commit in `backend` with message `Implement new feature`
- **AND** the command SHALL create one commit in `frontend` with message `Implement new feature`

#### Scenario: Skip non-Git child directories
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` has staged changes
- **AND** user runs `git vmr commit -m "Update backend"`
- **THEN** the command SHALL create one commit in `backend`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: Skip clean child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** only `backend` has staged changes
- **AND** user runs `git vmr commit -m "Update backend"`
- **THEN** the command SHALL create one commit in `backend`
- **AND** the command SHALL NOT create a commit in `frontend`

### Requirement: Commit command requires an explicit message option
The `git vmr commit` command SHALL require a commit message through `-m <message>` or `--message <message>`.

#### Scenario: Commit with short message option
- **WHEN** user runs `git vmr commit -m "Implement new feature"`
- **THEN** the command SHALL use `Implement new feature` as the commit message for each created commit

#### Scenario: Commit with long message option
- **WHEN** user runs `git vmr commit --message "Implement new feature"`
- **THEN** the command SHALL use `Implement new feature` as the commit message for each created commit

#### Scenario: Commit without message is rejected
- **WHEN** user runs `git vmr commit`
- **THEN** the command SHALL fail before attempting any child repository commit

### Requirement: Commit command reports concise repository-suffixed results
For each successful child repository commit, the `git vmr commit` command SHALL write the first non-empty line of Git's commit stdout to stdout with the child repository name appended in parentheses.

#### Scenario: Successful commits print Git summary lines with repository suffixes
- **WHEN** `backend` and `frontend` both commit successfully for message `Implement new feature`
- **AND** Git prints `[new-feature abcd123] Implement new feature` for `backend`
- **AND** Git prints `[new-feature 4567efg] Implement new feature` for `frontend`
- **THEN** stdout SHALL contain `[new-feature abcd123] Implement new feature (backend)`
- **AND** stdout SHALL contain `[new-feature 4567efg] Implement new feature (frontend)`

#### Scenario: Initial commit summary is preserved
- **WHEN** `backend` creates an initial commit
- **AND** Git prints `[main (root-commit) abcd123] Initial import`
- **THEN** stdout SHALL contain `[main (root-commit) abcd123] Initial import (backend)`

### Requirement: Commit command attempts every eligible repository and aggregates failures
The `git vmr commit` command SHALL attempt a commit in every child Git repository that has staged changes even if one or more eligible repositories fail, and SHALL exit with a non-zero status if any attempted commit fails.

#### Scenario: Failure does not stop later repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** all three repositories have staged changes
- **AND** committing `backend` fails
- **AND** committing `frontend` and `tools` can succeed
- **AND** user runs `git vmr commit -m "Implement new feature"`
- **THEN** the command SHALL attempt commits in `backend`, `frontend`, and `tools`
- **AND** the command SHALL create commits in `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Failed commits are reported with repository suffixes
- **WHEN** committing `backend` fails with Git error `pre-commit hook declined`
- **AND** user runs `git vmr commit -m "Implement new feature"`
- **THEN** stderr SHALL contain `pre-commit hook declined (backend)`

#### Scenario: Failed commit reports are deterministic
- **WHEN** committing fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr commit -m "Implement new feature"`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Commit command reports when nothing is eligible to commit
If no immediate child Git repository has staged changes, the `git vmr commit` command SHALL fail without attempting any commits and SHALL report that there is nothing to commit.

#### Scenario: No repositories have staged changes
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** neither repository has staged changes
- **AND** user runs `git vmr commit -m "Implement new feature"`
- **THEN** the command SHALL fail
- **AND** stderr SHALL contain `nothing to commit`

### Requirement: Commit command uses existing VMR working directory behavior
The `git vmr commit` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr commit -m "Implement new feature"` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and commit staged changes across eligible immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend commit -m "Implement new feature"`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and commit staged changes across eligible immediate child Git repositories
