## ADDED Requirements

### Requirement: Skipped outcomes are aggregated without failing
Fan-out Git commands SHALL support a skipped repo outcome carrying a required reason: the operation was deliberately not attempted. Skipped outcomes SHALL be rendered on stdout with repository attribution, grouped by identical reason under the existing aggregation rules, and SHALL NOT affect the exit code. Quiet-success behavior SHALL NOT apply to skips: a run whose outcomes are all skipped SHALL exit successfully and SHALL NOT be silent.

#### Scenario: Identical skip reasons are grouped on stdout
- **WHEN** a fan-out command skips child repositories `frontend` and `docs` with the same reason
- **THEN** stdout SHALL contain that reason exactly once
- **AND** stdout SHALL identify `frontend` and `docs` as the skipped repositories

#### Scenario: Skips do not change the exit code
- **WHEN** a fan-out command succeeds in child repository `backend` and skips child repository `docs`
- **THEN** the command SHALL exit successfully

#### Scenario: All-skipped run is not silent
- **WHEN** a fan-out command skips every child repository in scope
- **THEN** the command SHALL exit successfully
- **AND** stdout SHALL contain the skip reason or reasons
