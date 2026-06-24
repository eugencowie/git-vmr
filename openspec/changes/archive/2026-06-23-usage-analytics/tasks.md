## 1. Configuration

- [x] 1.1 Add `Analytics` global config with `enabled: Option<bool>`.
- [x] 1.2 Implement the version-based default: unset config enables analytics for `0.x` versions and disables it for `1.0.0` and later.
- [x] 1.3 Add config serialization and parsing tests for missing, enabled, and disabled analytics settings.

## 2. Command Metadata

- [x] 2.1 Add app-owned `CommandEvent` data with command, success, duration, flags, and global flags.
- [x] 2.2 Add stable command identifiers for all command variants, including `worktree.*` subcommands.
- [x] 2.3 Add parsed-command flag-name extraction that includes flag names only and excludes all values and operands.
- [x] 2.4 Add tests for command identifiers, command flags, global flags, and value redaction examples.

## 3. Analytics Delivery

- [x] 3.1 Use the published crates.io `aptabase-rs` dependency.
- [x] 3.2 Add an internal analytics module that records app-owned events through Aptabase without exposing Aptabase types to command parsing.
- [x] 3.3 Source the Aptabase app key in a way that missing or invalid keys make analytics no-op quietly.
- [x] 3.4 Flush one event without polling and wrap delivery in a short outer timeout.
- [x] 3.5 Add tests proving delivery failures and timeouts do not affect stdout, stderr, or exit status.

## 4. CLI Integration

- [x] 4.1 Change `Cli::run` to preserve the command result, record analytics after dispatch, run update checks only after success, and return the original command result.
- [x] 4.2 Add integration tests for analytics on successful dispatched commands.
- [x] 4.3 Add integration tests for analytics on failed dispatched commands.
- [x] 4.4 Add integration tests proving help, version, parse errors, and context construction failures do not record analytics.

## 5. Documentation

- [x] 5.1 Document that usage analytics are enabled by default before `1.0.0` when unset.
- [x] 5.2 Document `[analytics] enabled = false` as the opt-out.
- [x] 5.3 Document app-authored analytics fields and known `aptabase-rs` SDK-authored fields.

## 6. Release Readiness

- [x] 6.1 Switch `aptabase-rs` from the Git dependency to the published crates.io dependency before release.
