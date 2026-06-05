## ADDED Requirements

### Requirement: Add supports all-mode staging
The `git vmr add` command SHALL accept `-A` and `--all` to stage additions, modifications, and removals using Git all-mode semantics in each affected child repository.

#### Scenario: Stage all repositories without pathspecs
- **WHEN** user runs `git vmr add -A` from inside a VMR containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL stage `.` with `-A` in both `backend` and `frontend`

#### Scenario: Stage all repositories with long flag
- **WHEN** user runs `git vmr add --all` from inside a VMR containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL stage `.` with `--all` in both `backend` and `frontend`

#### Scenario: Stage routed pathspecs with all-mode
- **WHEN** user runs `git vmr add -A backend/src frontend/app` from the VMR root
- **THEN** the command SHALL stage `src` with `-A` in `backend` and `app` with `-A` in `frontend`

#### Scenario: Reject no pathspecs without all-mode
- **WHEN** user runs `git vmr add` without `-A`, `--all`, or pathspecs
- **THEN** the command SHALL fail during argument parsing without staging any child repository

### Requirement: Add supports force staging ignored paths
The `git vmr add` command SHALL accept `-f` and `--force` and SHALL pass the force option to each underlying `git add` invocation after VMR path routing succeeds.

#### Scenario: Stage ignored file with short force flag
- **WHEN** user runs `git vmr add -f backend/generated.log` and `generated.log` is ignored by the `backend` repository
- **THEN** the command SHALL stage `generated.log` in `backend`

#### Scenario: Stage ignored file with long force flag
- **WHEN** user runs `git vmr add --force backend/generated.log` and `generated.log` is ignored by the `backend` repository
- **THEN** the command SHALL stage `generated.log` in `backend`

### Requirement: Add supports chmod executable-bit updates
The `git vmr add` command SHALL accept `--chmod=+x` and `--chmod=-x` and SHALL pass the chmod option to each underlying `git add` invocation after VMR path routing succeeds.

#### Scenario: Set executable bit in index
- **WHEN** user runs `git vmr add --chmod=+x backend/script.sh`
- **THEN** the command SHALL set the executable bit for `script.sh` in the `backend` repository index without requiring the working tree file mode to change

#### Scenario: Clear executable bit in index
- **WHEN** user runs `git vmr add --chmod=-x backend/script.sh`
- **THEN** the command SHALL clear the executable bit for `script.sh` in the `backend` repository index without requiring the working tree file mode to change

#### Scenario: Reject unsupported chmod value
- **WHEN** user runs `git vmr add --chmod=bad backend/script.sh`
- **THEN** the command SHALL fail during argument parsing without staging any child repository

### Requirement: Add flags preserve VMR path validation
The `git vmr add` command SHALL validate VMR ownership for all explicit pathspecs before invoking any `git add` operation, even when `-A`, `--all`, `-f`, `--force`, or `--chmod` is present.

#### Scenario: Force does not allow VMR metadata path
- **WHEN** user runs `git vmr add --force .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without staging any child repository

#### Scenario: All-mode with invalid pathspec fails before staging
- **WHEN** user runs `git vmr add -A backend/src docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without staging any child repository

#### Scenario: Chmod does not allow path outside VMR
- **WHEN** user runs `git vmr add --chmod=+x ../outside.sh` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without staging any child repository
