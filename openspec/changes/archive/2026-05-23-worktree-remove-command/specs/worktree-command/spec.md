## ADDED Requirements

### Requirement: Worktree remove removes linked aggregate VMR worktrees
The `git vmr worktree remove <worktree>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to remove one Git worktree for every immediate child Git repository at `<worktree>/<repo-name>`.

#### Scenario: Remove linked worktree for multiple child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL attempt to remove the Git worktree for `backend` at `../wt/backend`
- **AND** the command SHALL attempt to remove the Git worktree for `frontend` at `../wt/frontend`
- **AND** the command SHALL exit successfully when both Git worktree removals succeed
- **AND** `../wt/backend` and `../wt/frontend` SHALL no longer exist

#### Scenario: Non-Git child directories are skipped during removal
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** a linked child worktree exists at `../wt/backend`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL attempt to remove the Git worktree for `backend` at `../wt/backend`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** the command SHALL NOT attempt to remove `../wt/docs`

### Requirement: Worktree remove delegates safety checks to Git
The `git vmr worktree remove <worktree>` command SHALL pass child worktree paths to `git worktree remove` without pre-filtering repositories based on target existence, dirty worktree state, locked worktree state, submodule presence, main worktree status, or other Git-enforced conditions.

#### Scenario: Dirty worktree failure is handled by Git
- **WHEN** a linked child worktree at `../wt/backend` has uncommitted modifications that Git refuses to remove without force
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL attempt `git worktree remove ../wt/backend` in repository `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git worktree removal failure for `backend`
- **AND** `../wt/backend` SHALL remain in place

#### Scenario: Missing child worktree failure is handled by Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** no linked child worktree exists at `../wt/backend`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL attempt `git worktree remove ../wt/backend` in repository `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git worktree removal failure for `backend`

### Requirement: Worktree remove supports repeated force flags
The `git vmr worktree remove [-f|--force] <worktree>` command SHALL accept repeated force flags and SHALL pass the same number of force occurrences to every underlying `git worktree remove` invocation.

#### Scenario: Single force is forwarded
- **WHEN** a linked child worktree at `../wt/backend` has uncommitted modifications that Git allows removing with one force flag
- **AND** user runs `git vmr worktree remove --force ../wt`
- **THEN** the command SHALL invoke Git worktree removal for `backend` with one `-f` force flag
- **AND** `../wt/backend` SHALL no longer exist when Git succeeds

#### Scenario: Double force is forwarded
- **WHEN** a linked child worktree at `../wt/backend` is locked and Git requires two force flags to remove it
- **AND** user runs `git vmr worktree remove --force --force ../wt`
- **THEN** the command SHALL invoke Git worktree removal for `backend` with two `-f` force flags
- **AND** `../wt/backend` SHALL no longer exist when Git succeeds

#### Scenario: Repeated short force is counted
- **WHEN** a linked child worktree at `../wt/backend` is locked and Git requires two force flags to remove it
- **AND** user runs `git vmr worktree remove -ff ../wt`
- **THEN** the command SHALL invoke Git worktree removal for `backend` with two `-f` force flags
- **AND** `../wt/backend` SHALL no longer exist when Git succeeds

### Requirement: Worktree remove is best-effort across child repositories
The `git vmr worktree remove <worktree>` command SHALL attempt worktree removal in every discovered child Git repository even if one or more repositories fail, and SHALL leave successful child worktree removals in place when other repositories fail.

#### Scenario: Failure does not stop other removals
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** worktree removal fails in `backend`
- **AND** worktree removal can succeed in `frontend` and `tools`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL attempt worktree removal in `backend`, `frontend`, and `tools`
- **AND** the command SHALL remove successful child worktrees for `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful child removals are not rolled back
- **WHEN** worktree removal succeeds in `frontend`
- **AND** worktree removal fails in `backend`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** `../wt/frontend` SHALL remain removed
- **AND** `../wt/backend` SHALL remain in place
- **AND** the command SHALL exit with a non-zero status

### Requirement: Worktree remove cleans aggregate VMR marker after complete success
The `git vmr worktree remove <worktree>` command SHALL remove the aggregate `.gitvmr` marker and remove the aggregate worktree directory if it is empty only after all child Git worktree removals succeed. If any child removal fails, the command SHALL leave the aggregate `.gitvmr` marker in place.

#### Scenario: Successful aggregate removal cleans marker and empty directory
- **WHEN** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** the command SHALL remove `../wt/backend` and `../wt/frontend`
- **AND** `../wt/.gitvmr` SHALL no longer exist
- **AND** `../wt` SHALL no longer exist when it is otherwise empty

#### Scenario: Partial aggregate removal keeps marker
- **WHEN** linked child worktree removal succeeds for `frontend`
- **AND** linked child worktree removal fails for `backend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** `../wt/.gitvmr` SHALL remain in place
- **AND** `../wt/backend` SHALL remain discoverable under the aggregate worktree root

### Requirement: Worktree remove reports deterministic repository-suffixed results
For each successful child repository worktree removal, stdout SHALL contain the first non-empty line from Git stdout or Git stderr with the child repository name appended in parentheses when Git emits a removal message. For each failed child repository worktree removal, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses. Failure reports SHALL be deterministic by child repository name.

#### Scenario: Failed removals are grouped with repository suffixes
- **WHEN** worktree removal fails in child repositories `backend` and `frontend` with the same Git error
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** stderr SHALL contain the Git error with repository suffix `(backend, frontend)`

#### Scenario: Failed removal reports are deterministic
- **WHEN** worktree removal fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr worktree remove ../wt`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Worktree remove uses existing working directory behavior
The `git vmr worktree remove <worktree>` command SHALL interpret the global `-C <path>` option the same way as existing commands. It SHALL use the resolved working directory as the starting point for VMR root discovery and as the base for resolving relative aggregate worktree paths.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr worktree remove ../wt` from inside child repository `frontend`
- **AND** a `.gitvmr` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root
- **AND** the command SHALL resolve `../wt` relative to the effective working directory

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend worktree remove ../wt`
- **AND** `/workspace/vmr/.gitvmr` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the source VMR root
- **AND** the command SHALL resolve the aggregate worktree path as `/workspace/vmr/wt`

### Requirement: Worktree remove validates CLI shape
The `git vmr worktree remove` command SHALL require exactly one aggregate worktree path. Unsupported extra operands SHALL fail during CLI argument validation before any child repository worktree removals are attempted.

#### Scenario: Missing worktree path is rejected
- **WHEN** user runs `git vmr worktree remove`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree removals SHALL be attempted

#### Scenario: Extra operands are rejected
- **WHEN** user runs `git vmr worktree remove ../wt extra`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree removals SHALL be attempted
