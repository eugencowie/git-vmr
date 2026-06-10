## Why

Users who install `git-vmr` through the generated installers should learn when a
new release is available without having to remember a separate update command.
Because `git-vmr` is a frequently used CLI, the check needs to be automatic but
throttled so normal Git workflows are not slowed down or made noisy.

## What Changes

- Add an automatic update check during normal `git-vmr` command execution.
- Use `axoupdater` to query the release source associated with the installed
  `git-vmr` binary.
- Throttle update checks to once per day by default.
- Add a global user-level configuration file, outside `.gitvmr`, that controls
  the check frequency with humantime duration strings of at least 1 hour or
  `never`.
- Store update-check runtime state separately from project `.gitvmr` metadata.
- Treat update-check failures as non-fatal so the requested command remains the
  primary behavior.
- Do not add a `git vmr config --global` command in this change.

## Capabilities

### New Capabilities

- `update-checking`: Automatic, globally configurable update checks for the
  `git-vmr` executable during normal CLI operation.

### Modified Capabilities

- None.

## Impact

- Adds an `axoupdater` dependency and any supporting runtime dependency needed
  to perform the async update query from the synchronous CLI entrypoint.
- Adds global configuration and update-check state file handling under the
  user's home/config/state area rather than `.gitvmr`.
- Updates CLI startup or dispatch flow so ordinary command execution can invoke
  the throttled check.
- Adds tests for frequency parsing, config defaults, throttling behavior,
  non-fatal update-check failures, and notification rendering.
