## Purpose

Specify how fan-out Git command output is aggregated so repeated child repository messages are rendered once while preserving repository attribution, quiet success behavior, and failure exit semantics.

## Requirements

### Requirement: Fan-out Git output is grouped by message
Fan-out Git commands SHALL render each unique successful or failed Git message once per command invocation, grouping repositories that produced the same message.

#### Scenario: Identical successes are printed once
- **WHEN** a fan-out command succeeds in child repositories `backend`, `frontend`, and `tools`
- **AND** Git emits `Already up to date.` for each successful repository
- **THEN** stdout SHALL contain `Already up to date.` exactly once
- **AND** stdout SHALL identify that `backend`, `frontend`, and `tools` produced that message, either by grouped repository names or by an aggregate repository count

#### Scenario: Different successes are printed separately
- **WHEN** a fan-out command succeeds in child repository `backend` with Git message `Already up to date.`
- **AND** the same command succeeds in child repository `frontend` with Git message `Updating abc123..def456`
- **THEN** stdout SHALL contain `Already up to date.` once
- **AND** stdout SHALL contain `Updating abc123..def456` once
- **AND** each line SHALL identify the repository or repositories that produced that message

#### Scenario: Identical failures are printed once
- **WHEN** a fan-out command fails in child repositories `alpha` and `zeta`
- **AND** Git emits `'origin' does not appear to be a git repository` for both failures
- **THEN** stderr SHALL contain `'origin' does not appear to be a git repository` exactly once
- **AND** stderr SHALL identify both `alpha` and `zeta` as repositories that produced that failure

#### Scenario: Successes and failures with the same text do not merge
- **WHEN** a fan-out command succeeds in child repository `backend` with Git message `message`
- **AND** the same command fails in child repository `frontend` with Git message `message`
- **THEN** stdout SHALL contain `message` for `backend`
- **AND** stderr SHALL contain `message` for `frontend`

### Requirement: Fan-out Git output preserves quiet successes
Fan-out Git commands SHALL NOT render output for successful child repository operations when Git emits no meaningful stdout or stderr message for those operations.

#### Scenario: All successes are quiet
- **WHEN** a fan-out command succeeds in every discovered child Git repository
- **AND** Git emits no meaningful output for any successful repository
- **THEN** stdout SHALL be empty
- **AND** stderr SHALL be empty

#### Scenario: Quiet successes do not hide failures
- **WHEN** a fan-out command succeeds quietly in child repository `backend`
- **AND** the same command fails in child repository `frontend` with Git message `pathspec did not match any files`
- **THEN** stdout SHALL be empty
- **AND** stderr SHALL contain `pathspec did not match any files`
- **AND** stderr SHALL identify `frontend` as the repository that produced the failure

### Requirement: Fan-out failure output preserves selected Git messages
Fan-out Git command failure rendering SHALL preserve the selected Git message text without adding, removing, or normalizing a `fatal:` prefix. The renderer SHALL append repository attribution after the preserved message according to the existing aggregate repository suffix rules.

#### Scenario: Git fatal message is preserved
- **WHEN** a fan-out command fails in child repositories `alpha` and `zeta`
- **AND** Git emits `fatal: invalid reference: feature/auth` for both failures
- **THEN** stderr SHALL contain `fatal: invalid reference: feature/auth` exactly once
- **AND** stderr SHALL NOT contain `fatal: fatal: invalid reference: feature/auth`
- **AND** stderr SHALL identify both `alpha` and `zeta` as repositories that produced that failure

#### Scenario: Git non-fatal error prefix is preserved
- **WHEN** a fan-out command fails in child repository `backend`
- **AND** Git emits `error: branch 'feature/auth' not found`
- **THEN** stderr SHALL contain `error: branch 'feature/auth' not found (backend)`
- **AND** stderr SHALL NOT contain `fatal: error: branch 'feature/auth' not found (backend)`

### Requirement: Fan-out command exit status preserves failure semantics
Fan-out Git commands SHALL continue attempting all eligible child repositories and SHALL exit with a non-zero status when one or more child repository operations fail, regardless of output grouping.

#### Scenario: Later repositories are still attempted after an earlier failure
- **WHEN** a fan-out command fails in child repository `backend`
- **AND** the same command can succeed in child repositories `frontend` and `tools`
- **THEN** the command SHALL attempt the operation in `backend`, `frontend`, and `tools`
- **AND** the command SHALL report the grouped failure for `backend`
- **AND** the command SHALL report any non-empty grouped success output for `frontend` and `tools`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: All grouped successes still exit successfully
- **WHEN** a fan-out command succeeds in every eligible child Git repository
- **AND** successful Git messages are grouped in stdout
- **THEN** the command SHALL exit successfully
