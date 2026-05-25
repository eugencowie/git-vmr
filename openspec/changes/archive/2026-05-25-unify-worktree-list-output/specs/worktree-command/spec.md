## MODIFIED Requirements

### Requirement: Worktree list shows aggregate VMR worktrees
The `git vmr worktree list` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, read each child Git repository's worktree list, and render one unified entry for each aggregate VMR worktree root.

#### Scenario: List main aggregate worktree for multiple child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list the VMR root aggregate worktree once
- **AND** the listed aggregate worktree SHALL represent both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: List linked aggregate worktree
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list `../wt` once as an aggregate worktree
- **AND** the listed aggregate worktree SHALL represent both `backend` and `frontend`

#### Scenario: Non-Git child directories are skipped during listing
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL read worktree information for `backend`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** the command SHALL NOT report `docs` as a repository participant

### Requirement: Worktree list reports branch and detached HEAD state
For each listed aggregate VMR worktree, `git vmr worktree list` SHALL report the aggregate path and the branch currently checked out by each participating child worktree. If a participating child worktree has no branch, the command SHALL report its detached HEAD state. Branch-backed child worktrees SHALL be grouped by branch name and SHALL NOT be split into separate rendered states solely because their HEAD hashes differ.

#### Scenario: Shared branch is reported once
- **WHEN** linked child worktrees at `../wt/backend` and `../wt/frontend` are both checked out on branch `wt`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt` once
- **AND** the command SHALL report branch `wt` once for that aggregate worktree
- **AND** the branch report SHALL NOT require separate repository suffixes for `backend` and `frontend`

#### Scenario: Shared branch with different child HEAD hashes is reported once
- **WHEN** linked child worktrees at `../wt/backend` and `../wt/frontend` are both checked out on branch `wt`
- **AND** `../wt/backend` and `../wt/frontend` have different HEAD hashes
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt` once
- **AND** the command SHALL report branch `wt` once for that aggregate worktree
- **AND** the command SHALL NOT render separate branch reports for each HEAD hash

#### Scenario: Mixed branches are reported under one aggregate worktree
- **WHEN** linked child worktree `../wt/backend` is checked out on branch `backend-topic`
- **AND** linked child worktree `../wt/frontend` is checked out on branch `frontend-topic`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt` once
- **AND** the command SHALL report branch `backend-topic` for `backend`
- **AND** the command SHALL report branch `frontend-topic` for `frontend`

#### Scenario: Detached child worktree is reported under one aggregate worktree
- **WHEN** linked child worktree `../wt/backend` is detached at a commit
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt` once
- **AND** the command SHALL report detached HEAD state for `backend`

### Requirement: Worktree list reports partial aggregate coverage
When an aggregate VMR worktree exists for only some child Git repositories, `git vmr worktree list` SHALL list the aggregate worktree once and append the participating repository names to state that applies to only those repositories.

#### Scenario: Partial linked aggregate worktree is listed with repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** no linked child worktree exists at `../wt/tools`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list `../wt` once as an aggregate worktree
- **AND** the command SHALL identify `backend` and `frontend` as the participating repositories
- **AND** the command SHALL NOT identify `tools` as a participant for `../wt`

### Requirement: Worktree list output is deterministic
The `git vmr worktree list` command SHALL render aggregate worktrees in deterministic path order. Within an aggregate worktree, rendered branch and detached states SHALL use deterministic state order, and repository suffixes SHALL list repositories in deterministic repository name order.

#### Scenario: Aggregate worktrees are sorted by path
- **WHEN** aggregate linked worktrees exist at `../zeta` and `../alpha`
- **AND** user runs `git vmr worktree list`
- **THEN** the output SHALL list `../alpha` before `../zeta`

#### Scenario: Repository suffixes are sorted by name
- **WHEN** aggregate linked worktree `../wt` exists for child repositories `zeta` and `alpha`
- **AND** user runs `git vmr worktree list`
- **THEN** the repository suffix for `../wt` SHALL list `alpha` before `zeta`

#### Scenario: Mixed states are sorted deterministically
- **WHEN** aggregate linked worktree `../wt` has child repositories on multiple branches and at least one detached HEAD state
- **AND** user runs `git vmr worktree list`
- **THEN** branch and detached state reports under `../wt` SHALL appear in deterministic order
