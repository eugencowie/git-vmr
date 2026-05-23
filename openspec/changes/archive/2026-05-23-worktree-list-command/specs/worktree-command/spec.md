## ADDED Requirements

### Requirement: Worktree list shows aggregate VMR worktrees
The `git vmr worktree list` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, read each child Git repository's worktree list, and render aggregate VMR worktrees grouped by aggregate root path.

#### Scenario: List main aggregate worktree for multiple child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list the VMR root aggregate worktree
- **AND** the listed aggregate worktree SHALL represent both `backend` and `frontend`
- **AND** the command SHALL exit successfully

#### Scenario: List linked aggregate worktree
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list `../wt` as an aggregate worktree
- **AND** the listed aggregate worktree SHALL represent both `backend` and `frontend`

#### Scenario: Non-Git child directories are skipped during listing
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL read worktree information for `backend`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** the command SHALL NOT report `docs` as a repository participant

### Requirement: Worktree list filters non-aggregate child worktrees
The `git vmr worktree list` command SHALL render only worktrees that correspond to the discovered VMR root or to a marked aggregate VMR linked worktree whose child paths follow the `<aggregate-root>/<repo-name>` layout.

#### Scenario: Unmarked matching child worktree parent is omitted
- **WHEN** a child Git repository `backend` has a linked worktree at `../scratch/backend`
- **AND** `../scratch/.gitvmr` does not exist
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL NOT list `../scratch` as an aggregate VMR worktree

#### Scenario: Arbitrary child worktree path is omitted
- **WHEN** a child Git repository `backend` has a linked worktree at `../backend-only`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL NOT list `../backend-only` as an aggregate VMR worktree
- **AND** the command SHALL NOT treat the parent directory of `../backend-only` as an aggregate VMR worktree

### Requirement: Worktree list reports branch and detached HEAD state
For each listed aggregate VMR worktree, `git vmr worktree list` SHALL report the aggregate path and the branch currently checked out by each participating child worktree. If a participating child worktree has no branch, the command SHALL report its detached HEAD state.

#### Scenario: Shared branch is reported once
- **WHEN** linked child worktrees at `../wt/backend` and `../wt/frontend` are both checked out on branch `wt`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt`
- **AND** the command SHALL report branch `wt` for that aggregate worktree
- **AND** the branch report SHALL NOT require separate repository suffixes for `backend` and `frontend`

#### Scenario: Mixed branches are reported with repository suffixes
- **WHEN** linked child worktree `../wt/backend` is checked out on branch `backend-topic`
- **AND** linked child worktree `../wt/frontend` is checked out on branch `frontend-topic`
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt` with branch `backend-topic` for `backend`
- **AND** the command SHALL list aggregate path `../wt` with branch `frontend-topic` for `frontend`

#### Scenario: Detached child worktree is reported
- **WHEN** linked child worktree `../wt/backend` is detached at a commit
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list aggregate path `../wt`
- **AND** the command SHALL report detached HEAD state for `backend`

### Requirement: Worktree list reports partial aggregate coverage
When an aggregate VMR worktree exists for only some child Git repositories, `git vmr worktree list` SHALL list the aggregate worktree and append the participating repository names.

#### Scenario: Partial linked aggregate worktree is listed with repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** no linked child worktree exists at `../wt/tools`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree list`
- **THEN** the command SHALL list `../wt` as an aggregate worktree
- **AND** the command SHALL identify `backend` and `frontend` as the participating repositories
- **AND** the command SHALL NOT identify `tools` as a participant for `../wt`

### Requirement: Worktree list output is deterministic
The `git vmr worktree list` command SHALL render aggregate worktrees in deterministic path order, and repository suffixes SHALL list repositories in deterministic repository name order.

#### Scenario: Aggregate worktrees are sorted by path
- **WHEN** aggregate linked worktrees exist at `../zeta` and `../alpha`
- **AND** user runs `git vmr worktree list`
- **THEN** the output SHALL list `../alpha` before `../zeta`

#### Scenario: Repository suffixes are sorted by name
- **WHEN** aggregate linked worktree `../wt` exists for child repositories `zeta` and `alpha`
- **AND** user runs `git vmr worktree list`
- **THEN** the repository suffix for `../wt` SHALL list `alpha` before `zeta`

### Requirement: Worktree list uses existing working directory behavior
The `git vmr worktree list` command SHALL interpret the global `-C <path>` option the same way as existing commands. It SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr worktree list` from inside child repository `frontend`
- **AND** a `.gitvmr` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root
- **AND** the command SHALL list aggregate VMR worktrees for that root

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend worktree list`
- **AND** `/workspace/vmr/.gitvmr` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root
- **AND** the command SHALL list aggregate VMR worktrees for `/workspace/vmr`

### Requirement: Worktree list validates CLI shape
The `git vmr worktree list` command SHALL accept no operands and no list-specific options. Unsupported list options and extra operands SHALL fail during CLI argument validation before any child repository worktree lists are read.

#### Scenario: Extra operand is rejected
- **WHEN** user runs `git vmr worktree list extra`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree lists SHALL be read

#### Scenario: Porcelain option is rejected
- **WHEN** user runs `git vmr worktree list --porcelain`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree lists SHALL be read

#### Scenario: Null termination option is rejected
- **WHEN** user runs `git vmr worktree list -z`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree lists SHALL be read

#### Scenario: Verbose option is rejected
- **WHEN** user runs `git vmr worktree list -v`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree lists SHALL be read

## MODIFIED Requirements

### Requirement: Worktree add validates CLI shape
The `git vmr worktree add` command SHALL require exactly one aggregate target path and SHALL accept at most one optional commit-ish argument. Unsupported worktree subcommands and extra operands SHALL fail during CLI argument validation before any child repository worktree additions are attempted.

#### Scenario: Missing target path is rejected
- **WHEN** user runs `git vmr worktree add`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

#### Scenario: Extra operands are rejected
- **WHEN** user runs `git vmr worktree add ../wt main extra`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

#### Scenario: Unsupported worktree subcommand is rejected
- **WHEN** user runs `git vmr worktree lock ../wt`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted
