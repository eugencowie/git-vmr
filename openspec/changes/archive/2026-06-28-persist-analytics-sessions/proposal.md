## Why

Each CLI invocation currently creates a fresh Aptabase client without a caller-owned session ID, so events from commands run close together are reported as separate sessions. Persisting the session ID lets related command usage appear as one session while keeping analytics best-effort and anonymous.

## What Changes

- Persist an analytics session ID and last activity timestamp in the existing user-level state file.
- Reuse the persisted session ID for dispatched command analytics while activity remains within the Aptabase session window.
- Rotate to a new SDK-generated session ID after the session window expires.
- Pass the selected session ID to `aptabase-rs` when recording `command_finished`.
- Keep analytics disabled behavior, excluded command paths, delivery timeout behavior, and command payload privacy unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `usage-analytics`: Command analytics events should share a session ID across invocations within the session window.

## Impact

- Affects `src/analytics.rs`, `src/cli.rs`, and global state serialization in `src/state.rs`.
- Uses existing `aptabase-rs` caller-owned session ID API; no new dependency is needed.
- Updates usage analytics tests and documentation for persisted session behavior.
