## ADDED Requirements

### Requirement: Analytics session lifecycle
When usage analytics are enabled and a parsed command reaches command dispatch, the CLI SHALL assign the `command_finished` event to an analytics session that is persisted across CLI invocations. The CLI SHALL reuse the persisted session ID while the last recorded analytics activity is within the Aptabase session window, and SHALL generate a new session ID after that window expires. Analytics session state failures SHALL NOT change the command exit status or replace the original command result; they MAY emit the existing global state save warning.

#### Scenario: Consecutive dispatched commands reuse session
- **WHEN** user runs `git-vmr status`
- **AND** usage analytics are enabled
- **AND** user runs another dispatched command within the Aptabase session window
- **THEN** both `command_finished` analytics events SHALL use the same session ID

#### Scenario: Expired session rotates
- **WHEN** a persisted analytics session was last active before the Aptabase session window
- **AND** user runs `git-vmr status`
- **AND** usage analytics are enabled
- **THEN** the `command_finished` analytics event SHALL use a newly generated session ID
- **AND** the new session ID SHALL be persisted as the active analytics session

#### Scenario: Failed dispatched command updates session
- **WHEN** user runs a parsed command that reaches command dispatch and returns an error
- **AND** usage analytics are enabled
- **THEN** the CLI SHALL attempt one `command_finished` analytics event with the active session ID
- **AND** the active session last activity SHALL be updated
- **AND** the CLI SHALL save changed context data before returning
- **AND** the CLI SHALL return the original command error status

#### Scenario: Excluded command paths do not create session
- **WHEN** user runs clap help output, clap version output, a clap parse error, or a command that fails before command dispatch
- **THEN** the CLI SHALL NOT create or update analytics session state

## MODIFIED Requirements

### Requirement: Analytics documentation
The project documentation SHALL disclose that usage analytics are enabled by default before the `1.0.0` release when no explicit analytics configuration is present. The documentation SHALL show how to disable analytics, SHALL list all app-authored properties and known `aptabase-rs` SDK-authored fields sent with events, and SHALL describe that session IDs are reused across command invocations within the Aptabase session window.

#### Scenario: Documentation includes opt-out config
- **WHEN** user reads the analytics documentation
- **THEN** the documentation SHALL show `[analytics] enabled = false`

#### Scenario: Documentation lists analytics fields
- **WHEN** user reads the analytics documentation
- **THEN** the documentation SHALL list app-authored properties `name`, `success`, `duration_ms`, and boolean flag properties such as `force_flag` and `working_dir_global_flag`
- **AND** the documentation SHALL list known SDK-authored fields including app version, SDK version, operating system name, operating system version, locale, debug flag, timestamp, and session ID
- **AND** the documentation SHALL explain that session IDs are reused across command invocations within the Aptabase session window
