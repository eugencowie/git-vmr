## Context

`git-vmr` is a synchronous Rust CLI whose normal subcommands are dispatched
through `Cli::run`. The repository already has VMR-local TOML config under
`.gitvmr/config`, but there is no global user-level configuration model. The
release setup uses dist-generated shell and PowerShell installers, and
`axoupdater` can use the install receipt created by those installers to discover
the release source and current installation details.

Automatic update checks are user-visible but not core Git behavior. They should
therefore be quiet unless useful, non-fatal on update-query errors, and
throttled before network work begins.

## Goals / Non-Goals

**Goals:**

- Check for newer `git-vmr` releases automatically during normal successful CLI
  operation.
- Default checks to once per day.
- Let users configure update-check frequency with humantime duration strings or
  `never` by editing a global config file outside `.gitvmr`.
- Persist update-check runtime state outside project metadata.
- Avoid retry storms by recording an attempted check before the network query.
- Bound due update checks with a short timeout unless `axoupdater` already
  provides one.
- Preserve existing command stdout/stderr behavior except for update notices and
  fatal global config errors.

**Non-Goals:**

- Add `git vmr config --global` or any other config-editing command.
- Automatically install updates during ordinary command execution.
- Add an explicit `git vmr update` command.
- Support package-manager-specific update flows.
- Block, fail, or roll back the requested command when the update check fails.

## Decisions

### Load config before running normal commands

Load global config after argument parsing and before command dispatch. A missing
global config file uses defaults. An existing but malformed global config file,
including invalid values, is a fatal error and the requested subcommand does not
run. This keeps user-authored policy validation in the config module and avoids
mixing config error handling into update-check orchestration.

### Run checks after successful normal commands

Invoke the update check after `Cli::run` returns `Ok(())`. Help/version exits
are handled by clap before `Cli::run`, and failed subcommands should keep their
error output focused on the command failure.

Alternative considered: check before every command. That could delay simple
commands before doing useful work and could show update messages next to
unrelated errors.

### Store global config and runtime state in user-level paths

Use platform user directories instead of `.gitvmr`:

| Purpose | Path |
| --- | --- |
| Config | `<config_dir>/git-vmr/config.toml` |
| State | `<state_dir>/git-vmr/update.toml` |

Resolve these through a path helper so tests can inject temporary paths. On
platforms where no state directory is available, fall back to the local data
directory for update state. Missing config means the default frequency is
`daily`; missing state means a check is due.

Alternative considered: use `.gitvmr/config`. That would make update policy
project-local, but the installed executable and release cadence are global to
the user.

Alternative considered: use a single global file for config and state. Keeping
state separate avoids future `git vmr config --global` edits rewriting volatile
timestamps.

### Use a small global update config model

Add a dedicated global config type rather than extending the existing VMR
config type:

```toml
[updates]
check_frequency = "1 day"
```

Supported values are humantime duration strings of at least `1h`, such as
`1h`, `1 day`, `7 days`, and `30 days`, plus `never`.

Alternative considered: only support named frequencies such as `daily` and
`weekly`. Humantime duration strings provide more flexibility while remaining
readable in a hand-edited config file.

### Throttle before network work

When a check is due, write `last_attempted_check` to state before calling
`axoupdater`. This prevents repeated network attempts on every command when the
network, GitHub, the receipt, or the release source is unavailable.

If a newer version is found, store `last_available_version` and print a concise
notice to stderr. If no update is found, store the attempt timestamp only. If
the check fails, preserve the attempt timestamp and do not print an update
notice.

Alternative considered: write state only after a successful query. That keeps
state semantically cleaner but creates retry storms during outages.

### Query, do not install

Use `AxoUpdater::new_for("git-vmr")`, `load_receipt`, and `query_new_version`
or equivalent update-query APIs. Do not call `run`, because ordinary command
execution must not mutate the installed binary.

If the loaded receipt is not for the running executable, skip the network query.
This avoids showing installer-based update messages for a `cargo install`,
package-manager, or manually copied executable when an unrelated dist receipt
exists.

Alternative considered: configure the GitHub release source manually and query
without receipts. That would reach more installation methods, but it would also
encourage update notices for installations that should be managed elsewhere.

### Use a short timeout for due checks

Due checks should not be able to hang ordinary command usage for an unbounded
period. If `axoupdater` already enforces a short request timeout, rely on that
default. If it does not, configure its client or wrap the query future so each
due update check has a short timeout.

Alternative considered: rely entirely on OS/network defaults. That leaves a
successful `git-vmr` command vulnerable to unexpectedly long post-command
delays.

### Keep notices version-only for now

When a newer release is found, print only a concise notice containing the
available version. Do not include project URLs or installer commands in this
change.

Alternative considered: include the project site or installer command in the
notice. That can be added later, but keeping the first notice short minimizes
noise in normal command output.

### Keep update-check errors non-fatal

Update checks should not return errors to `main`. Receipt, eligibility, network,
release parsing, state-write, and state-read errors are ignored except for
best-effort state writes needed for throttling.

Alternative considered: treat invalid global config as a warning and suppress
update checking for that invocation. Failing before command dispatch makes
hand-edited user policy errors visible and keeps config validation owned by the
config module.

## Risks / Trade-offs

- [Synchronous commands can be delayed by a due network check] -> Throttle
  checks, run them after command success, and use a short timeout if
  `axoupdater` does not provide one.
- [Users may not know how to install the available update] -> Keep the initial
  notice concise and leave installer guidance for future UX work.
- [Malformed global config can block normal commands] -> Fail before command
  dispatch with the parse error so the user can fix the hand-edited config.
- [Tests involving user directories can be flaky] -> Implement path resolution
  behind injectable helpers and test with temporary paths.
- [Package-manager installs may not receive notices] -> Deliberately limit
  automatic checks to receipt-backed dist installs until package-manager update
  policy is designed.

## Migration Plan

1. Add the global update config and state models without creating config files
   automatically.
2. Add the update-check module and wire it after successful command execution.
3. Add `axoupdater` and the minimal runtime support needed to call its async
   query API.
4. Add tests for default frequency, configured durations, `never`, due/not-due
   state, state update before query, non-fatal failures, and notification
   output.
5. Validate normal command behavior still matches existing tests.

Rollback consists of removing the post-command update-check call. Any config or
state files left in user directories are inert if the code no longer reads them.
