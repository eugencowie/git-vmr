## MODIFIED Requirements

### Requirement: Switch command reports repository-suffixed results
For each successful child repository switch, stdout SHALL contain the selected Git success line from Git stdout or Git stderr with repository attribution rendered according to aggregate output grouping. Before grouping successful switch output, the command SHALL normalize Git behind-and-fast-forward advisory lines by removing the variable `by N commit` or `by N commits` phrase. For each failed child repository switch, stderr SHALL contain the first non-empty line from Git stderr, or Git stdout if stderr is empty, with repository attribution rendered according to aggregate output grouping.

#### Scenario: Successful switches print Git summary lines with repository suffixes
- **WHEN** switching to `feature/auth` succeeds in `backend`
- **AND** Git prints `Switched to branch 'feature/auth'`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stdout SHALL contain `Switched to branch 'feature/auth' (backend)`

#### Scenario: Behind counts are normalized before grouping
- **WHEN** switching to `develop` succeeds in child Git repositories `backend` and `frontend`
- **AND** Git prints `Your branch is behind 'origin/develop' by 20 commits, and can be fast-forwarded.` for `backend`
- **AND** Git prints `Your branch is behind 'origin/develop' by 6 commits, and can be fast-forwarded.` for `frontend`
- **AND** user runs `git vmr switch develop`
- **THEN** stdout SHALL contain `Your branch is behind 'origin/develop', and can be fast-forwarded.` exactly once
- **AND** stdout SHALL identify both `backend` and `frontend` according to aggregate output grouping
- **AND** stdout SHALL NOT contain `by 20 commits`
- **AND** stdout SHALL NOT contain `by 6 commits`

#### Scenario: Failed switches are reported with repository suffixes
- **WHEN** switching to `feature/auth` fails in `backend` with Git error `invalid reference: feature/auth`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stderr SHALL contain `invalid reference: feature/auth (backend)`

#### Scenario: Failed switch reports are deterministic
- **WHEN** switching fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr switch feature/auth`
- **THEN** stderr SHALL report the `alpha` failure before the `zeta` failure
