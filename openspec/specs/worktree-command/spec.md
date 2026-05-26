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

### Requirement: Worktree add supports explicit branch creation
The `git vmr worktree add -b <new-branch> <path> [<commit-ish>]` command SHALL create each child repository worktree on the requested new branch by passing `-b <new-branch>` to Git. When `<commit-ish>` is provided, the command SHALL pass it to Git as the start point for the new branch. When `<commit-ish>` is omitted, Git SHALL use its default start point behavior. The command SHALL NOT infer a branch name from the aggregate target path when `-b <new-branch>` is provided.

#### Scenario: Explicit branch without commit-ish
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add -b feature/auth ../wt`
- **THEN** the command SHALL attempt to create a Git worktree for `backend` using new branch `feature/auth` at `../wt/backend`
- **AND** the command SHALL attempt to create a Git worktree for `frontend` using new branch `feature/auth` at `../wt/frontend`
- **AND** the command SHALL NOT create branch `wt` from the aggregate target path basename

#### Scenario: Explicit branch with commit-ish start point
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add -b feature/auth ../wt main`
- **THEN** the command SHALL attempt `git worktree add -b feature/auth` with explicit commit-ish `main` in both repositories
- **AND** the created child worktrees SHALL be checked out on branch `feature/auth` when Git worktree addition succeeds

#### Scenario: Explicit branch creation failure is reported per repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** branch `feature/auth` already exists in both repositories
- **AND** user runs `git vmr worktree add -b feature/auth ../wt`
- **THEN** the command SHALL attempt explicit branch worktree addition in both repositories
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git branch creation failure for `backend` and `frontend`

#### Scenario: Branch flag requires branch name
- **WHEN** user runs `git vmr worktree add -b`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

#### Scenario: Long branch alias is rejected
- **WHEN** user runs `git vmr worktree add --branch feature/auth ../wt`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

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
- **WHEN** user runs `git vmr worktree lock ../wt`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

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

### Requirement: Worktree list renders Git-style path separators
The `git vmr worktree list` command SHALL render aggregate worktree paths with Git-style `/` separators, regardless of the separator style used by the underlying platform path display or Git worktree list input.

#### Scenario: Windows-style aggregate path is normalized
- **WHEN** `git vmr worktree list` renders an aggregate worktree path represented as `C:\Projects\vmr`
- **THEN** the output SHALL contain `C:/Projects/vmr`
- **AND** the output SHALL NOT contain `C:\Projects\vmr`

#### Scenario: Mixed path sources render consistently
- **WHEN** `git vmr worktree list` renders aggregate worktree paths represented as `C:\Projects\vmr` and `C:/Worktrees/new-feature`
- **THEN** the output SHALL contain `C:/Projects/vmr`
- **AND** the output SHALL contain `C:/Worktrees/new-feature`
- **AND** both rendered paths SHALL use `/` as the path separator

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
