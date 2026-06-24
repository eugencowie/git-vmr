## Why

Pre-1.0 `git-vmr` needs lightweight usage data to understand which commands and flags are used, where commands fail, and what should be improved before the 1.0 release. The data must avoid project-sensitive values and must never make CLI commands slower or less reliable.

## What Changes

- Add usage analytics for normal command invocations using `aptabase-rs`.
- Enable analytics by default only while `git-vmr` is below `1.0.0`; keep an explicit global config override.
- Track one quiet `command_finished` event after command execution for both successful and failed commands.
- Track command name, success state, duration, and used flag names as boolean properties.
- Do not track flag values, paths, repository names, remotes, branch names, commit messages, shell commands, commit identifiers, or error text.
- Keep analytics delivery timeout-bound and non-fatal.
- Document pre-1.0 default-on behavior, opt-out configuration, and all app-authored and SDK-authored fields.
- Use a Git dependency for `aptabase-rs` initially, then switch to the published crates.io dependency before release.

## Capabilities

### New Capabilities
- `usage-analytics`: Defines opt-in/opt-out behavior, event timing, event payload, privacy constraints, and delivery behavior for CLI usage analytics.

### Modified Capabilities

## Impact

- Affected code: CLI command runner, command/flag metadata extraction, global configuration, analytics delivery module, tests, and documentation.
- Dependencies: temporary Git dependency on `eugencowie/aptabase-rs`; final implementation task switches to a published crates.io dependency.
- Systems: Aptabase event ingestion for pre-1.0 usage analytics.
