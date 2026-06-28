## Purpose
Define best-effort CLI usage analytics configuration, command event payloads, delivery behavior, and documentation.

## Requirements

### Requirement: Analytics default and configuration
The CLI SHALL read usage analytics configuration from the user-level global configuration file. When `[analytics] enabled` is unset, usage analytics SHALL be enabled by default for package versions lower than `1.0.0` and disabled by default for package versions `1.0.0` and higher. An explicit `[analytics] enabled = true` or `[analytics] enabled = false` value SHALL override the version-based default.

#### Scenario: Missing analytics config before 1.0 enables analytics
- **WHEN** the running package version is `0.9.1`
- **AND** no `[analytics] enabled` value is configured
- **THEN** usage analytics SHALL be enabled

#### Scenario: Missing analytics config from 1.0 disables analytics
- **WHEN** the running package version is `1.0.0`
- **AND** no `[analytics] enabled` value is configured
- **THEN** usage analytics SHALL be disabled

#### Scenario: Explicit analytics opt-out wins before 1.0
- **WHEN** the running package version is `0.9.1`
- **AND** the global configuration contains `[analytics] enabled = false`
- **THEN** usage analytics SHALL be disabled

#### Scenario: Explicit analytics opt-in wins from 1.0
- **WHEN** the running package version is `1.0.0`
- **AND** the global configuration contains `[analytics] enabled = true`
- **THEN** usage analytics SHALL be enabled

### Requirement: Command completion events
The CLI SHALL attempt to record a single `command_finished` usage analytics event after each parsed command invocation that reaches command dispatch. The event SHALL be attempted for commands that complete successfully and for commands that return an error after dispatch. The CLI SHALL NOT attempt usage analytics for clap help output, clap version output, clap parse errors, global configuration load failures, global state load failures, or CLI context construction failures.

#### Scenario: Successful dispatched command records event
- **WHEN** user runs `git-vmr status`
- **AND** the command reaches dispatch and completes successfully
- **THEN** the CLI SHALL attempt one `command_finished` analytics event
- **AND** the event property `success` SHALL be `true`

#### Scenario: Failed dispatched command records event
- **WHEN** user runs `git-vmr status`
- **AND** the command reaches dispatch and returns an error
- **THEN** the CLI SHALL attempt one `command_finished` analytics event
- **AND** the event property `success` SHALL be `false`

#### Scenario: Version output does not record event
- **WHEN** user runs `git-vmr --version`
- **THEN** the CLI SHALL NOT attempt a usage analytics event

#### Scenario: Context construction failure does not record event
- **WHEN** user runs `git-vmr -C missing status`
- **AND** resolving the effective working directory fails before command dispatch
- **THEN** the CLI SHALL NOT attempt a usage analytics event

### Requirement: Analytics event payload
The `command_finished` event SHALL include only app-authored properties for command identifier, success state, duration in milliseconds, and used flag names serialized as boolean properties. Command flags SHALL use `<name>_flag` properties, and global flags SHALL use `<name>_global_flag` properties. The command identifier SHALL be stable and SHALL include nested subcommands where applicable. The CLI SHALL NOT include raw argument strings, flag values, operands, paths, repository names, remote names, branch names, commit identifiers, commit messages, shell commands, or error text in app-authored analytics properties.

#### Scenario: Boolean and value flags record names only
- **WHEN** user runs `git-vmr rm --force --dry-run path/to/file`
- **THEN** the analytics event SHALL include `force_flag = true` and `dry_run_flag = true`
- **AND** the analytics event SHALL NOT include `path/to/file`

#### Scenario: Global working directory flag records name only
- **WHEN** user runs `git-vmr -C /private/project status`
- **THEN** the analytics event SHALL include `working_dir_global_flag = true`
- **AND** the analytics event SHALL NOT include `/private/project`

#### Scenario: Worktree subcommand uses stable nested identifier
- **WHEN** user runs `git-vmr worktree remove --force ../feature`
- **THEN** the analytics event command identifier SHALL be `worktree.remove`
- **AND** the analytics event SHALL include `force_flag = true`
- **AND** the analytics event SHALL NOT include `../feature`

#### Scenario: Foreach command does not record shell command
- **WHEN** user runs `git-vmr foreach --quiet echo secret`
- **THEN** the analytics event SHALL include `quiet_flag = true`
- **AND** the analytics event SHALL NOT include `echo secret`

### Requirement: Analytics delivery behavior
Usage analytics delivery SHALL use `aptabase-rs` as a best-effort, timeout-bound delivery mechanism. Analytics delivery failures, timeouts, missing app keys, invalid app keys, network errors, and Aptabase errors SHALL NOT change command stdout, stderr, or exit status. Analytics delivery SHALL NOT run as a background polling loop for normal CLI invocations.

#### Scenario: Analytics failure is non-fatal
- **WHEN** a dispatched command completes successfully
- **AND** analytics delivery fails
- **THEN** the command SHALL still exit successfully
- **AND** analytics delivery failure details SHALL NOT be printed to stdout or stderr

#### Scenario: Analytics timeout is non-fatal
- **WHEN** analytics delivery exceeds the configured analytics timeout
- **THEN** the command SHALL continue without waiting for the Aptabase request timeout
- **AND** analytics timeout details SHALL NOT be printed to stdout or stderr

#### Scenario: Failed command preserves original failure
- **WHEN** a dispatched command returns an error
- **AND** analytics delivery succeeds or fails
- **THEN** the CLI SHALL return the original command error

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
