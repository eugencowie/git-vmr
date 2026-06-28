## 1. State

- [x] 1.1 Add analytics session state to `GlobalState` with persisted session ID and last activity timestamp.
- [x] 1.2 Implement session selection that reuses IDs within four hours and rotates expired or missing sessions with `aptabase_rs::new_session_id()`.
- [x] 1.3 Add state serialization and lifecycle tests for missing, active, expired, and failed-save-safe session state.

## 2. Analytics Recording

- [x] 2.1 Select and mark the analytics session from the analytics recording path.
- [x] 2.2 Build the Aptabase client with `Builder::with_session_id(...)`.
- [x] 2.3 Save changed context data after analytics recording so failed dispatched commands refresh the session before returning.
- [x] 2.4 Keep `sessionId` out of app-authored analytics props and `GITVMR_ANALYTICS_LOG` props output.

## 3. Verification

- [x] 3.1 Add integration tests proving consecutive dispatched commands reuse a session ID.
- [x] 3.2 Add integration tests proving failed dispatched commands update and reuse the session without changing the original failure.
- [x] 3.3 Add tests proving excluded command paths do not create analytics session state.
- [x] 3.4 Update analytics documentation to describe session ID reuse within the Aptabase session window.
- [x] 3.5 Run `cargo test analytics` and the relevant state tests.
