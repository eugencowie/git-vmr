## 1. Dependencies and Module Structure

- [x] 1.1 Add `axoupdater` and the minimal supporting crates needed for async query execution, user-directory resolution, and timestamp serialization.
- [x] 1.2 Add an update-check module with separate types for global update config, update-check state, check frequency, and path resolution.
- [x] 1.3 Keep the existing `.gitvmr` config model separate from the new global update config model.

## 2. Global Config and State

- [x] 2.1 Implement loading global update config from the user-level config path with `daily` as the missing-file default.
- [x] 2.2 Implement parsing and validation for `hourly`, `daily`, `weekly`, `monthly`, and `never`.
- [x] 2.3 Implement update-check state loading and saving from the user-level state path outside `.gitvmr`.
- [x] 2.4 Implement frequency-based due checks where `monthly` is treated as 30 days.
- [x] 2.5 Record the last attempted check before any network query when a check is due.

## 3. Axoupdater Query

- [x] 3.1 Implement a query path that uses `AxoUpdater::new_for("git-vmr")` and the installed receipt for release-source and version information.
- [x] 3.2 Skip the release-source query when the installed receipt is missing or does not belong to the running executable.
- [x] 3.3 Query for a newer version without installing updates, using a short timeout if `axoupdater` does not already enforce one.
- [x] 3.4 Render a concise stderr notice only when a newer version is available, including only the available version.
- [x] 3.5 Treat timeout, receipt, eligibility, state, network, and release-source errors as non-fatal to the requested command.

## 4. CLI Wiring

- [x] 4.1 Invoke the throttled update check after successful normal subcommand execution.
- [x] 4.2 Ensure failed subcommands, help output, and version output do not trigger update checks.
- [x] 4.3 Preserve existing command stdout and stderr behavior except for update notices and any non-fatal global config warning.

## 5. Tests

- [x] 5.1 Add unit tests for global config defaults, supported frequency parsing, unsupported frequency handling, and `never`.
- [x] 5.2 Add unit tests for state loading, state saving, due/not-due interval calculations, and attempt-time recording before query.
- [x] 5.3 Add tests for eligible receipt, ineligible receipt, missing receipt, newer-version, current-version, timeout, and query-failure outcomes using an injectable updater abstraction.
- [x] 5.4 Add CLI-level tests that successful subcommands can trigger a due check while failed subcommands, help, and version output do not.
- [x] 5.5 Add regression coverage showing update-check failures do not fail otherwise successful commands.

## 6. Documentation and Validation

- [x] 6.1 Document the global update-check config file and supported `check_frequency` values.
- [x] 6.2 Run formatting checks.
- [x] 6.3 Run Clippy checks.
- [x] 6.4 Run the test suite.
