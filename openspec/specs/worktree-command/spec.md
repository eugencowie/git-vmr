## Purpose
Specify behavior for linked aggregate VMR worktree creation across child Git repositories.

## Requirements

### Requirement: Worktree add creates linked aggregate VMR worktrees
The `git vmr worktree add <path> [<commit-ish>]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to create one Git worktree for every immediate child Git repository under the aggregate target path.

#### Scenario: Add linked worktree for multiple child repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL attempt to create a Git worktree for `backend` at `../wt/backend`
- **AND** the command SHALL attempt to create a Git worktree for `frontend` at `../wt/frontend`
- **AND** the command SHALL exit successfully when both Git worktree additions succeed

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL attempt to create a Git worktree for `backend` at `../wt/backend`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** the command SHALL NOT create `../wt/docs`

### Requirement: Worktree add infers aggregate branch when commit-ish is omitted
When `<commit-ish>` is omitted, `git vmr worktree add <path>` SHALL derive a new branch name from the basename of the aggregate target path and SHALL create each child repository worktree on that same new branch by passing the inferred branch name explicitly to Git.

#### Scenario: Omitted commit-ish creates same branch in every child worktree
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL derive branch name `wt` from aggregate target path `../wt`
- **AND** `../wt/backend` SHALL be created on new branch `wt`
- **AND** `../wt/frontend` SHALL be created on new branch `wt`

#### Scenario: Inferred branch creation failure is reported per repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** branch `wt` already exists in both repositories
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL attempt branch-inferred worktree addition in both repositories
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git branch creation failure for `backend` and `frontend`

### Requirement: Worktree add delegates explicit commit-ish resolution to Git
When `<commit-ish>` is provided, `git vmr worktree add <path> <commit-ish>` SHALL pass the commit-ish directly to `git worktree add` in each child repository without creating a branch derived from the aggregate target path.

#### Scenario: Explicit branch already checked out fails through Git
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** branch `main` is already checked out in the source worktree for both repositories
- **AND** user runs `git vmr worktree add ../wt main`
- **THEN** the command SHALL attempt `git worktree add` with explicit commit-ish `main` in both repositories
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL contain `fatal: 'main' is already used by worktree` for `backend` and `frontend`
- **AND** no branch named `wt` SHALL be created by the VMR command

#### Scenario: Invalid explicit commit-ish fails through Git
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `new` is not a valid Git reference in either repository
- **AND** user runs `git vmr worktree add ../wt new`
- **THEN** the command SHALL attempt `git worktree add` with explicit commit-ish `new` in both repositories
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL contain `fatal: invalid reference: new` for `backend` and `frontend`
- **AND** no branch named `wt` SHALL be created by the VMR command

### Requirement: Worktree add is best-effort across child repositories
The `git vmr worktree add <path> [<commit-ish>]` command SHALL attempt worktree addition in every discovered child Git repository even if one or more repositories fail, and SHALL leave successful child worktrees in place when other repositories fail.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** worktree addition fails in `backend`
- **AND** worktree addition can succeed in `frontend` and `tools`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL attempt worktree addition in `backend`, `frontend`, and `tools`
- **AND** the command SHALL create successful child worktrees for `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful child worktrees are not rolled back
- **WHEN** worktree addition succeeds in `frontend`
- **AND** worktree addition fails in `backend`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** `../wt/frontend` SHALL remain in place
- **AND** the command SHALL exit with a non-zero status

### Requirement: Worktree add creates VMR marker in aggregate target
The `git vmr worktree add <path> [<commit-ish>]` command SHALL create a `.gitvmr` marker at the aggregate target path so VMR root discovery works from inside the linked worktree.

#### Scenario: Linked worktree contains VMR marker
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr worktree add ../wt`
- **AND** the `backend` worktree is created successfully at `../wt/backend`
- **THEN** `../wt/.gitvmr` SHALL exist

#### Scenario: Commands discover linked aggregate root from child worktree
- **WHEN** a linked aggregate worktree exists at `../wt`
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr status` from `../wt/backend`
- **THEN** VMR root discovery SHALL use `../wt` as the aggregate VMR root

### Requirement: Worktree add reports deterministic repository-suffixed results
For each successful child repository worktree addition, stdout SHALL contain the first non-empty line from Git stdout or Git stderr with the child repository name appended in parentheses. For each failed child repository worktree addition, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses. Failure reports SHALL be deterministic by child repository name.

#### Scenario: Successful additions print repository-suffixed Git summary lines
- **WHEN** worktree addition succeeds in child repository `backend`
- **AND** Git prints `Preparing worktree (new branch 'wt')`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** stdout SHALL contain `Preparing worktree (new branch 'wt') (backend)`

#### Scenario: Failed additions are grouped with repository suffixes
- **WHEN** worktree addition fails in child repositories `backend` and `frontend` with Git error `fatal: invalid reference: new`
- **AND** user runs `git vmr worktree add ../wt new`
- **THEN** stderr SHALL contain `fatal: invalid reference: new (backend, frontend)`

#### Scenario: Failed addition reports are deterministic
- **WHEN** worktree addition fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr worktree add ../wt new`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Worktree add uses existing working directory behavior
The `git vmr worktree add <path> [<commit-ish>]` command SHALL interpret the global `-C <path>` option the same way as existing commands. It SHALL use the resolved working directory as the starting point for VMR root discovery and as the base for resolving relative aggregate target paths.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr worktree add ../wt` from inside child repository `frontend`
- **AND** a `.gitvmr` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root
- **AND** the command SHALL resolve `../wt` relative to the effective working directory

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend worktree add ../wt`
- **AND** `/workspace/vmr/.gitvmr` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the source VMR root
- **AND** the command SHALL resolve the aggregate target path as `/workspace/vmr/wt`

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
- **WHEN** user runs `git vmr worktree list`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted
