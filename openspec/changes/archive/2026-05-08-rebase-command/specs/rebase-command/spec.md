## ADDED Requirements

### Requirement: Rebase command attempts rebases across child repositories
The `git vmr rebase <upstream>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and attempt to rebase every immediate child Git repository onto the requested upstream.

#### Scenario: Clean rebase in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both repositories can rebase onto `origin/main`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL rebase both `backend` and `frontend` onto `origin/main`
- **AND** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** `backend` can rebase onto `origin/main`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL rebase `backend` onto `origin/main`
- **AND** the command SHALL NOT fail because of `docs`
- **AND** stdout and stderr SHALL be empty

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Rebase command delegates upstream resolution to Git
The `git vmr rebase <upstream>` command SHALL pass the provided upstream to `git rebase` in each child repository without pre-filtering repositories based on local branch or remote-tracking ref existence.

#### Scenario: Missing upstream fails only that repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` can rebase onto `origin/main`
- **AND** `frontend` cannot resolve `origin/main`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL attempt the rebase in both repositories
- **AND** the command SHALL rebase `backend` onto `origin/main`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `frontend` failure

#### Scenario: Detached commit can be used as upstream
- **WHEN** a child repository can rebase onto commit hash `abc1234`
- **AND** user runs `git vmr rebase abc1234`
- **THEN** the command SHALL pass `abc1234` to Git as the rebase upstream

### Requirement: Rebase command is best-effort across repositories
The `git vmr rebase <upstream>` command SHALL attempt a rebase in every discovered child Git repository even if one or more repositories fail, and SHALL exit with a non-zero status if any attempted rebase fails.

#### Scenario: Failure does not stop other repositories
- **WHEN** a VMR contains child Git repositories `backend`, `frontend`, and `tools`
- **AND** rebasing onto `origin/main` fails in `backend`
- **AND** rebasing onto `origin/main` can succeed in `frontend` and `tools`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL attempt rebases in `backend`, `frontend`, and `tools`
- **AND** the command SHALL rebase `frontend` and `tools` onto `origin/main`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Successful rebases are not rolled back after failure
- **WHEN** rebasing onto `origin/main` succeeds in `backend`
- **AND** rebasing onto `origin/main` fails in `frontend`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the successful rebase in `backend` SHALL remain applied
- **AND** the command SHALL exit with a non-zero status

### Requirement: Rebase command runs child repository attempts in parallel
The `git vmr rebase <upstream>` command SHALL run child repository rebase attempts in parallel while preserving deterministic aggregate reporting.

#### Scenario: Parallel attempts still produce deterministic failures
- **WHEN** rebasing fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Rebase conflicts are preserved for user resolution
When Git stops a rebase because of conflicts in a child repository, the `git vmr rebase <upstream>` command SHALL treat that repository as failed and SHALL leave the repository in Git's normal in-progress rebase state.

#### Scenario: Conflicted repository remains in rebase state
- **WHEN** rebasing `backend` onto `origin/main` produces conflicts
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL exit with a non-zero status
- **AND** `backend` SHALL remain in an in-progress rebase state
- **AND** stderr SHALL report the `backend` rebase failure

#### Scenario: Conflict does not stop clean repositories
- **WHEN** rebasing `backend` onto `origin/main` produces conflicts
- **AND** rebasing `frontend` onto `origin/main` can succeed
- **AND** user runs `git vmr rebase origin/main`
- **THEN** the command SHALL leave `backend` in an in-progress rebase state
- **AND** the command SHALL rebase `frontend` onto `origin/main`
- **AND** the command SHALL exit with a non-zero status

### Requirement: Rebase command reports only repository-suffixed failures
For each failed child repository rebase, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with the child repository name appended in parentheses. Successful rebases SHALL NOT produce aggregate output.

#### Scenario: Successful rebases are quiet
- **WHEN** rebasing `origin/main` succeeds in every discovered child Git repository
- **AND** user runs `git vmr rebase origin/main`
- **THEN** stdout and stderr SHALL be empty

#### Scenario: Failed rebases are reported with repository suffixes
- **WHEN** rebasing `origin/main` fails in `backend` with Git error `invalid upstream 'origin/main'`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** stderr SHALL contain `invalid upstream 'origin/main' (backend)`

#### Scenario: Failed rebase reports are deterministic
- **WHEN** rebasing fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr rebase origin/main`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure

### Requirement: Rebase command uses existing VMR working directory behavior
The `git vmr rebase <upstream>` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr rebase origin/main` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and attempt rebases across immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend rebase origin/main`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and attempt rebases across immediate child Git repositories
