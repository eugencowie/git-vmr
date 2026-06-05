## ADDED Requirements

### Requirement: Worktree move moves linked aggregate VMR worktrees
The `git vmr worktree move <worktree> <new-path>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to move one Git worktree for every immediate child Git repository from `<worktree>/<repo-name>` to `<new-path>/<repo-name>`.

#### Scenario: Move linked worktree for multiple child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt to move the Git worktree for `backend` from `../wt/backend` to `../moved/backend`
- **AND** the command SHALL attempt to move the Git worktree for `frontend` from `../wt/frontend` to `../moved/frontend`
- **AND** the command SHALL exit successfully when both Git worktree moves succeed
- **AND** `../moved/backend` and `../moved/frontend` SHALL exist
- **AND** `../wt/backend` and `../wt/frontend` SHALL no longer exist

#### Scenario: Non-Git child directories are skipped during move
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** a linked child worktree exists at `../wt/backend`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt to move the Git worktree for `backend` from `../wt/backend` to `../moved/backend`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** the command SHALL NOT attempt to move `../wt/docs`

### Requirement: Worktree move delegates safety checks to Git
The `git vmr worktree move <worktree> <new-path>` command SHALL pass child worktree source and destination paths to `git worktree move` without pre-filtering repositories based on source existence, destination existence, dirty worktree state, locked worktree state, submodule presence, main worktree status, or other Git-enforced conditions.

#### Scenario: Dirty worktree failure is handled by Git
- **WHEN** a linked child worktree at `../wt/backend` has uncommitted modifications that Git refuses to move without force
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt `git worktree move ../wt/backend ../moved/backend` in repository `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git worktree move failure for `backend`
- **AND** `../wt/backend` SHALL remain in place

#### Scenario: Missing child worktree failure is handled by Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** no linked child worktree exists at `../wt/backend`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt `git worktree move ../wt/backend ../moved/backend` in repository `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git worktree move failure for `backend`

#### Scenario: Existing destination child failure is handled by Git
- **WHEN** a linked child worktree exists at `../wt/backend`
- **AND** `../moved/backend` already exists
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt `git worktree move ../wt/backend ../moved/backend` in repository `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git worktree move failure for `backend`

### Requirement: Worktree move supports repeated force flags
The `git vmr worktree move [-f|--force] <worktree> <new-path>` command SHALL accept repeated force flags and SHALL pass the same number of force occurrences to every underlying `git worktree move` invocation.

#### Scenario: Single force is forwarded
- **WHEN** a linked child worktree at `../wt/backend` has uncommitted modifications that Git allows moving with one force flag
- **AND** user runs `git vmr worktree move --force ../wt ../moved`
- **THEN** the command SHALL invoke Git worktree move for `backend` with one `-f` force flag
- **AND** `../moved/backend` SHALL exist when Git succeeds

#### Scenario: Repeated long force is counted
- **WHEN** a linked child worktree at `../wt/backend` requires two force flags to move
- **AND** user runs `git vmr worktree move --force --force ../wt ../moved`
- **THEN** the command SHALL invoke Git worktree move for `backend` with two `-f` force flags

#### Scenario: Repeated short force is counted
- **WHEN** a linked child worktree at `../wt/backend` requires two force flags to move
- **AND** user runs `git vmr worktree move -ff ../wt ../moved`
- **THEN** the command SHALL invoke Git worktree move for `backend` with two `-f` force flags

### Requirement: Worktree move is best-effort across child repositories
The `git vmr worktree move <worktree> <new-path>` command SHALL attempt worktree moves in every discovered child Git repository even if one or more repositories fail, and SHALL leave successful child worktree moves in place when other repositories fail.

#### Scenario: Failure does not stop other moves
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** worktree move fails in `backend`
- **AND** worktree move can succeed in `frontend` and `tools`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL attempt worktree move in `backend`, `frontend`, and `tools`
- **AND** the command SHALL move successful child worktrees for `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful child moves are not rolled back
- **WHEN** worktree move succeeds in `frontend`
- **AND** worktree move fails in `backend`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** `../moved/frontend` SHALL remain in place
- **AND** `../wt/frontend` SHALL no longer exist
- **AND** `../wt/backend` SHALL remain in place
- **AND** the command SHALL exit with a non-zero status

### Requirement: Worktree move maintains aggregate VMR markers
The `git vmr worktree move <worktree> <new-path>` command SHALL create the destination aggregate directory and destination `.gitvmr` marker before child worktree moves are attempted. It SHALL remove the source aggregate `.gitvmr` marker and remove the source aggregate worktree directory if it is empty only after all child Git worktree moves succeed. If any child move fails, the command SHALL leave both source and destination aggregate markers in place.

#### Scenario: Successful aggregate move transfers discoverability
- **WHEN** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** the command SHALL create `../moved/.gitvmr`
- **AND** the command SHALL move `../wt/backend` and `../wt/frontend` to `../moved/backend` and `../moved/frontend`
- **AND** `../wt/.gitvmr` SHALL no longer exist
- **AND** `../wt` SHALL no longer exist when it is otherwise empty

#### Scenario: Partial aggregate move keeps both markers
- **WHEN** linked child worktree move succeeds for `frontend`
- **AND** linked child worktree move fails for `backend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** `../wt/.gitvmr` SHALL remain in place
- **AND** `../moved/.gitvmr` SHALL remain in place
- **AND** `../wt/backend` SHALL remain discoverable under the source aggregate worktree root
- **AND** `../moved/frontend` SHALL remain discoverable under the destination aggregate worktree root

#### Scenario: Total aggregate move failure leaves destination marker
- **WHEN** linked child worktree moves fail for every child Git repository
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** `../wt/.gitvmr` SHALL remain in place
- **AND** `../moved/.gitvmr` SHALL remain in place

### Requirement: Worktree move reports deterministic repository-suffixed results
For each successful child repository worktree move, stdout SHALL contain the first non-empty line from Git stdout or Git stderr with the child repository name appended when Git emits a move message. For each failed child repository worktree move, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended. Failure reports SHALL be deterministic by child repository name.

#### Scenario: Failed moves are grouped with repository suffixes
- **WHEN** worktree move fails in child repositories `backend` and `frontend` with the same Git error
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** stderr SHALL contain the Git error with repository suffix `(backend, frontend)`

#### Scenario: Failed move reports are deterministic
- **WHEN** worktree move fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr worktree move ../wt ../moved`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Worktree move uses existing working directory behavior
The `git vmr worktree move <worktree> <new-path>` command SHALL interpret the global `-C <path>` option the same way as existing commands. It SHALL use the resolved working directory as the starting point for VMR root discovery and as the base for resolving relative aggregate source and destination paths.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr worktree move ../wt ../moved` from inside child repository `frontend`
- **AND** a `.gitvmr` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root
- **AND** the command SHALL resolve `../wt` and `../moved` relative to the effective working directory

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend worktree move ../wt ../moved`
- **AND** `/workspace/vmr/.gitvmr` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the source VMR root
- **AND** the command SHALL resolve the aggregate source path as `/workspace/vmr/wt`
- **AND** the command SHALL resolve the aggregate destination path as `/workspace/vmr/moved`

### Requirement: Worktree move validates CLI shape
The `git vmr worktree move` command SHALL require exactly one aggregate source worktree path and exactly one aggregate destination path. Unsupported extra operands SHALL fail during CLI argument validation before any child repository worktree moves are attempted.

#### Scenario: Missing source worktree path is rejected
- **WHEN** user runs `git vmr worktree move`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree moves SHALL be attempted

#### Scenario: Missing destination path is rejected
- **WHEN** user runs `git vmr worktree move ../wt`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree moves SHALL be attempted

#### Scenario: Extra operands are rejected
- **WHEN** user runs `git vmr worktree move ../wt ../moved extra`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree moves SHALL be attempted
