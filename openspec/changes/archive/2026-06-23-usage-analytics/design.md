## Context

`git-vmr` already has a single post-command side-effect path for update checks after successful command execution. Usage analytics needs a similar quiet path, but it must also run after command failures that happen after CLI parsing and context creation. Analytics must not observe raw arguments because raw arguments can include paths, branches, remotes, commit messages, shell commands, and other project-specific data.

`aptabase-rs` currently exists as a GitHub repository dependency rather than a crates.io crate. The change can use that Git dependency during implementation, but release readiness requires switching to the published crate.

## Goals / Non-Goals

**Goals:**
- Track one `command_finished` event for parsed command invocations that reach command dispatch.
- Include success/failure, duration, command identity, and used flag names as boolean properties.
- Keep pre-1.0 analytics enabled by default when config is unset, with an explicit user override.
- Avoid sensitive values by deriving telemetry from parsed command structures, not raw argv.
- Keep delivery quiet, timeout-bound, and non-fatal.
- Keep the app-owned event shape separate from Aptabase so a durable queue can be added later.

**Non-Goals:**
- Track clap parse failures, help output, version output, or context/config load failures.
- Track flag values, operands, paths, branch names, remotes, commit IDs, commit messages, shell commands, repository names, or error text.
- Add a durable queue in this change.
- Add panic tracking.

## Decisions

### Use an app-owned `CommandEvent`

Create an internal event struct with fields for command name, success, duration, command flags, and global flags. The analytics module serializes each flag as a boolean property such as `force_flag` or `working_dir_global_flag`. `Cli::run` records that event through a small analytics module; command parsing and command execution do not depend on Aptabase types.

Alternative considered: call `aptabase-rs` directly from `Cli::run`. That is less code today, but it couples command metadata to a vendor payload and makes a later file-backed queue harder.

### Track after command execution before update checks

`Cli::run` should store the command result instead of immediately returning with `?`, record analytics quietly, run update checks only when the command succeeded, then return the original command result.

Alternative considered: keep the current `?` flow and track only successes. That misses the failure data needed before 1.0.

### Derive flags from parsed command state

Each command variant should expose a stable command identifier and a list of used flag names. Boolean flags are included when true. Value flags are included by name only when present. Operands are never included. The analytics payload flattens those names into boolean properties for dashboard filtering instead of sending arrays. Worktree subcommands should use subcommand identifiers such as `worktree.list` and `worktree.remove`.

Alternative considered: sanitize raw argv. That is brittle and easier to get wrong.

### Use config `Option<bool>` for the default flip

Add `[analytics] enabled = true|false` to global config. Missing `enabled` uses `env!("CARGO_PKG_VERSION").starts_with("0.")`: enabled before 1.0.0, disabled from 1.0.0 onward. This avoids a semver dependency for one simple cutoff.

Alternative considered: store an explicit default value in generated config. That would make the 1.0 behavior depend on old files instead of the running version.

### Keep delivery best-effort

Use `aptabase-rs` without polling, enqueue one event, and flush it with a short outer timeout. Analytics failures, missing app keys, invalid app keys, network errors, and timeout errors must not affect command output or exit status.

Alternative considered: durable delivery now. That adds state-file complexity before there is evidence the extra reliability matters.

### Leave a durable queue path

The analytics module should own serialization of `CommandEvent` independently of Aptabase. If event loss matters later, `analytics::record(event)` can append events to a state-root file, flush oldest events, and remove flushed events without changing command metadata extraction.

## Risks / Trade-offs

- Default-on telemetry can reduce user trust before 1.0 -> document it plainly, provide a global opt-out, and keep payloads minimal.
- Aptabase SDK adds system fields beyond app-authored properties -> document both app-authored and SDK-authored fields.
- A short timeout can drop events on slow networks -> acceptable for pre-1.0 usage analytics; durable queue can be added later.
- Git dependency is not release-clean -> keep a final implementation task to switch to the published crates.io crate.
