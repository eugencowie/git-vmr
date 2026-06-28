## Context

Usage analytics currently records one `command_finished` event after each dispatched command. `src/analytics.rs` builds a new Aptabase client for each CLI process without supplying a session ID, so the SDK generates a fresh session for every invocation.

`aptabase-rs` already exposes `new_session_id()` and `Builder::with_session_id(...)`. The app only needs to decide which session ID to use and persist enough state to reuse it later.

## Goals / Non-Goals

**Goals:**

- Reuse one analytics session ID across CLI invocations within the Aptabase session window.
- Persist only the session ID and last activity timestamp in the existing global state file.
- Preserve existing analytics exclusions, payload privacy, timeouts, and failure behavior.
- Keep failed dispatched commands in the same analytics session because they already emit analytics events.

**Non-Goals:**

- Add a configurable session timeout.
- Add a new analytics store or durable event queue.
- Add cross-process state locking.
- Change command event properties or collect argument values.

## Decisions

1. Store session state in `GlobalState`.

   Add an analytics state section beside the existing update state. This reuses the existing user-level state path and serialization behavior instead of creating another file.

   `analytics::record` may update this state through `CliContext` because it is the only analytics dispatch point and already owns the choice of whether an analytics event will be recorded.

2. Match Aptabase's four-hour session window.

   Reuse the persisted session ID when `last_activity` is within four hours, otherwise generate a new one with `aptabase_rs::new_session_id()`. This keeps app-owned persistence aligned with SDK behavior.

   Alternative considered: add a user config value. There is no product need for tuning this, and it would expose analytics internals.

3. Update session state while recording analytics, then save through the normal CLI lifecycle.

   `analytics::record` should select the session, update the session activity, mark global state dirty, and use the selected session ID when building the Aptabase client. `Cli::run` should save changed context data after analytics recording even when the dispatched command returns an error, then return the original command result.

   State save failures may print the existing warning, but must not replace the original command result or change the exit status.

4. Keep session ID out of app-authored properties.

   `analytics::record` should build the Aptabase client with `Builder::with_session_id(session_id)`. The SDK sends `sessionId` as event envelope metadata, so the app-authored `props` object and `GITVMR_ANALYTICS_LOG` props test hook should not include `sessionId`.

## Risks / Trade-offs

- Concurrent CLI invocations can race on the shared state file -> accept last-writer-wins for now; add locking only if parallel usage causes real session churn.
- State save failure can prevent reuse -> keep it non-fatal; the existing warning is acceptable, but the original command result and exit status must be preserved.
- Persisting a session ID adds local state -> the ID is already sent with analytics events and contains no command arguments or repository data.
