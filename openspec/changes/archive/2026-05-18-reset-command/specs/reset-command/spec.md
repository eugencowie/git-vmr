## ADDED Requirements

### Requirement: Reset command attempts resets across child repositories
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to reset every immediate child Git repository.

#### Scenario: Reset in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can reset to `HEAD~1`
- **AND** user runs `git vmr reset HEAD~1`
- **THEN** the command SHALL reset both `backend` and `frontend` to `HEAD~1`
- **AND** the command SHALL exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can reset to `HEAD~1`
- **AND** user runs `git vmr reset HEAD~1`
- **THEN** the command SHALL reset `backend` to `HEAD~1`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr reset HEAD~1`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Reset command supports reset modes
The `git vmr reset` command SHALL accept at most one reset mode from `--soft`, `--mixed`, `--hard`, `--merge`, and `--keep`, and SHALL pass the selected mode to `git reset` in each child repository.

#### Scenario: Soft reset mode is forwarded
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --soft HEAD~1`
- **THEN** the command SHALL run `git reset --soft HEAD~1` in both `backend` and `frontend`

#### Scenario: Mixed reset mode is forwarded
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --mixed HEAD~1`
- **THEN** the command SHALL run `git reset --mixed HEAD~1` in both `backend` and `frontend`

#### Scenario: Hard reset mode is forwarded
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --hard HEAD~1`
- **THEN** the command SHALL run `git reset --hard HEAD~1` in both `backend` and `frontend`

#### Scenario: Merge reset mode is forwarded
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --merge HEAD~1`
- **THEN** the command SHALL run `git reset --merge HEAD~1` in both `backend` and `frontend`

#### Scenario: Keep reset mode is forwarded
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --keep HEAD~1`
- **THEN** the command SHALL run `git reset --keep HEAD~1` in both `backend` and `frontend`

#### Scenario: Reset modes are mutually exclusive
- **WHEN** user runs `git vmr reset --soft --hard HEAD~1`
- **THEN** argument parsing SHALL fail
- **AND** no child repository reset SHALL be attempted

### Requirement: Reset command supports optional commit
The `git vmr reset` command SHALL pass the optional commit argument to `git reset` in each child repository when provided, and SHALL omit the commit argument when it is not provided.

#### Scenario: Reset with commit
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset HEAD~1`
- **THEN** the command SHALL run `git reset HEAD~1` in both `backend` and `frontend`

#### Scenario: Reset without commit
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset`
- **THEN** the command SHALL run `git reset` in both `backend` and `frontend`

#### Scenario: Reset with mode and no commit
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr reset --hard`
- **THEN** the command SHALL run `git reset --hard` in both `backend` and `frontend`

### Requirement: Reset command delegates repository-state resolution to Git
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL pass reset arguments to Git without pre-filtering repositories based on commit existence, branch state, index state, working tree cleanliness, merge state, or reset mode safety.

#### Scenario: Missing commit fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` can resolve `release-base`
- **AND** `frontend` cannot resolve `release-base`
- **AND** user runs `git vmr reset release-base`
- **THEN** the command SHALL attempt the reset in both repositories
- **AND** the command SHALL reset `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Dirty working tree behavior is delegated to Git
- **WHEN** a child repository has local changes that prevent `git reset --keep HEAD~1`
- **AND** user runs `git vmr reset --keep HEAD~1`
- **THEN** the command SHALL attempt `git reset --keep HEAD~1` in that repository
- **AND** the command SHALL exit with a non-zero status if Git rejects the reset

### Requirement: Reset command is best-effort across repositories
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL attempt a reset in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted reset fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** resetting to `release-base` fails in `backend`
- **AND** resetting to `release-base` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr reset release-base`
- **THEN** the command SHALL attempt resets in `backend`, `frontend`, and `tools`
- **AND** the command SHALL reset `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful resets are not rolled back after failure
- **WHEN** resetting to `HEAD~1` succeeds in `backend`
- **AND** resetting to `HEAD~1` fails in `frontend`
- **AND** user runs `git vmr reset HEAD~1`
- **THEN** the successful reset in `backend` SHALL remain applied
- **AND** the command SHALL exit with a non-zero status

### Requirement: Reset command runs child repository attempts in parallel
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL run child repository reset attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** resetting fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr reset missing-ref`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Reset command reports repository-suffixed results
For each successful child repository reset that emits output, stdout SHALL contain the first non-empty line from Git stdout, or Git stderr if stdout is empty, with the child repository name appended. Successful child repository resets that emit no output SHALL NOT produce aggregate output. For each failed reset, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended.

#### Scenario: Successful hard reset prints Git summary line with repository suffix
- **WHEN** resetting `backend` succeeds and Git emits `HEAD is now at abc1234 update` on stdout
- **AND** user runs `git vmr reset --hard HEAD~1`
- **THEN** stdout SHALL contain `HEAD is now at abc1234 update (backend)`

#### Scenario: Successful reset with no Git output is quiet
- **WHEN** resetting every discovered child Git repository succeeds with no Git stdout or stderr output
- **AND** user runs `git vmr reset`
- **THEN** stdout and stderr SHALL be empty

#### Scenario: Failed reset reports repository suffix
- **WHEN** resetting to `missing-ref` fails in `backend` with Git error `fatal: ambiguous argument 'missing-ref'`
- **AND** user runs `git vmr reset missing-ref`
- **THEN** stderr SHALL contain `fatal: ambiguous argument 'missing-ref' (backend)`

#### Scenario: Failed reset reports are deterministic
- **WHEN** resetting fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr reset missing-ref`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Reset command rejects unsupported pathspec and extra arguments
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL reject unsupported extra positional arguments and SHALL NOT support pathspec reset forms.

#### Scenario: Extra positional argument is rejected
- **WHEN** user runs `git vmr reset HEAD~1 extra`
- **THEN** argument parsing SHALL fail
- **AND** no child repository reset SHALL be attempted

#### Scenario: Pathspec separator is rejected
- **WHEN** user runs `git vmr reset HEAD -- backend/file.txt`
- **THEN** argument parsing SHALL fail
- **AND** no child repository reset SHALL be attempted

### Requirement: Reset command uses existing VMR working directory behavior
The `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr reset HEAD~1` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt resets across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend reset HEAD~1`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt resets across its immediate child Git repositories
