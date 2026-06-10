## ADDED Requirements

### Requirement: Automatic update check during normal operation
The `git-vmr` CLI SHALL perform an automatic update check after a normal
subcommand completes successfully when the configured check frequency indicates
that a check is due. The update check SHALL NOT run for clap help or version
output, and SHALL NOT run after a subcommand exits with an error.

#### Scenario: Successful command triggers due check
- **WHEN** user runs a `git-vmr` subcommand that completes successfully
- **AND** update checks are configured to run daily
- **AND** no update-check state exists
- **THEN** the CLI SHALL attempt an update check after the subcommand completes

#### Scenario: Failed command does not trigger check
- **WHEN** user runs a `git-vmr` subcommand that exits with an error
- **THEN** the CLI SHALL NOT attempt an update check for that invocation

#### Scenario: Version output does not trigger check
- **WHEN** user runs `git-vmr --version`
- **THEN** the CLI SHALL print the version without attempting an update check

### Requirement: Global update configuration
The system SHALL read update-check configuration from a global user-level
configuration file outside `.gitvmr`. When the global configuration file is
missing, the system SHALL use `daily` as the default update-check frequency.
The supported frequency values SHALL be humantime duration strings of at least
1 hour and `never`.

#### Scenario: Missing global config uses daily
- **WHEN** no global `git-vmr` configuration file exists
- **THEN** the update-check frequency SHALL be `daily`

#### Scenario: Configured duration frequency is honored
- **WHEN** the global `git-vmr` configuration file contains
  `[updates] check_frequency = "7 days"`
- **THEN** the update-check frequency SHALL be 7 days

#### Scenario: Never disables update checks
- **WHEN** the global `git-vmr` configuration file contains
  `[updates] check_frequency = "never"`
- **THEN** the CLI SHALL NOT attempt automatic update checks

#### Scenario: Invalid global config is fatal
- **WHEN** the global `git-vmr` configuration file contains an unsupported
  update-check frequency
- **AND** user runs a `git-vmr` subcommand
- **THEN** the CLI SHALL report the config parse error
- **AND** the CLI SHALL NOT run the requested subcommand
- **AND** the CLI SHALL NOT attempt an update check for that invocation

#### Scenario: Syntax error in global config is fatal
- **WHEN** the global `git-vmr` configuration file contains invalid TOML syntax
- **AND** user runs a `git-vmr` subcommand
- **THEN** the CLI SHALL report the config parse error
- **AND** the CLI SHALL NOT run the requested subcommand
- **AND** the CLI SHALL NOT attempt an update check for that invocation

#### Scenario: Sub-hour duration is invalid
- **WHEN** the global `git-vmr` configuration file contains
  `[updates] check_frequency = "30 minutes"`
- **AND** user runs a `git-vmr` subcommand
- **THEN** the CLI SHALL report the config parse error
- **AND** the CLI SHALL NOT run the requested subcommand
- **AND** the CLI SHALL NOT attempt an update check for that invocation

### Requirement: Update checks are throttled by state
The system SHALL persist update-check runtime state outside `.gitvmr` and SHALL
use the state to avoid checking more often than the configured humantime
duration. Configured durations shorter than 1 hour SHALL be rejected.

#### Scenario: Recent daily check skips network query
- **WHEN** update checks are configured to run daily
- **AND** update-check state records an attempted check less than 1 day ago
- **AND** user runs a `git-vmr` subcommand that completes successfully
- **THEN** the CLI SHALL NOT query the update source

#### Scenario: Expired daily check queries update source
- **WHEN** update checks are configured to run daily
- **AND** update-check state records an attempted check more than 1 day ago
- **AND** user runs a `git-vmr` subcommand that completes successfully
- **THEN** the CLI SHALL query the update source

#### Scenario: Due check records attempt before query
- **WHEN** an automatic update check is due
- **AND** user runs a `git-vmr` subcommand that completes successfully
- **THEN** the system SHALL record the current time as the last attempted check
  before querying the update source

### Requirement: Update queries use axoupdater receipt eligibility
The automatic update check SHALL use `axoupdater` with the `git-vmr` application
name and SHALL use the installed receipt to determine whether the running
executable is eligible for installer-based update checks. The automatic check
SHALL query for a newer version only when the receipt belongs to the running
executable.

#### Scenario: Eligible receipt permits query
- **WHEN** an automatic update check is due
- **AND** the installed receipt belongs to the running `git-vmr` executable
- **THEN** the CLI SHALL query the release source for a newer version

#### Scenario: Ineligible receipt skips query
- **WHEN** an automatic update check is due
- **AND** the installed receipt does not belong to the running `git-vmr`
  executable
- **THEN** the CLI SHALL NOT query the release source for a newer version

#### Scenario: Missing receipt is non-fatal
- **WHEN** an automatic update check is due
- **AND** no compatible installed receipt can be loaded
- **AND** user runs a `git-vmr` subcommand that otherwise completes successfully
- **THEN** the subcommand SHALL still succeed
- **AND** the CLI SHALL NOT print an update notice

### Requirement: Update notices are informational
The automatic update check SHALL NOT install updates. When a newer version is
available, the CLI SHALL print a concise notice to stderr after the successful
subcommand output. The notice SHALL include the available version and SHALL NOT
include the current version, project URLs, or installer commands. When no newer
version is available, the CLI SHALL NOT print an update notice.

#### Scenario: New version prints notice
- **WHEN** an automatic update check finds a newer `git-vmr` version
- **THEN** the CLI SHALL print an update notice to stderr
- **AND** the notice SHALL include the available version
- **AND** the notice SHALL NOT include the current version, project URLs, or
  installer commands

#### Scenario: Current version is quiet
- **WHEN** an automatic update check finds no newer `git-vmr` version
- **THEN** the CLI SHALL NOT print an update notice

#### Scenario: Update check failure is quiet and non-fatal
- **WHEN** an automatic update check encounters a receipt, state, network, or
  release-source error
- **AND** user runs a `git-vmr` subcommand that otherwise completes successfully
- **THEN** the subcommand SHALL still succeed
- **AND** the CLI SHALL NOT print an update notice
