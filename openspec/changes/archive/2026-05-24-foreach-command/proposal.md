## Why

Users working in a virtual monorepo need a quick way to run the same ad hoc command in every child repository without replacing the VMR layout with Git submodules. Today this requires external shell loops that each user has to write, quote, order, and error-handle themselves.

## What Changes

- Add `git vmr foreach <command>` for running an arbitrary shell command in every immediate child Git repository.
- Discover repositories through the existing VMR root and immediate-child repository model, skipping non-Git children.
- Run child commands in parallel for performance, then render each repository's buffered output sequentially in deterministic repository order.
- Run each command with the child repository as its working directory and expose VMR-aware environment variables for scripts.
- Add `--quiet` to suppress repository entry headers.
- Exit successfully only when every child command exits successfully; still wait for and render all child command results when one or more commands fail.

## Capabilities

### New Capabilities
- `foreach-command`: Runs arbitrary shell commands across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new top-level CLI subcommand and command module.
- Uses existing VMR discovery and repository scanning behavior.
- Adds a shell-command execution path with buffered stdout, buffered stderr, exit status capture, and deterministic rendering.
- Adds integration tests for command execution, output ordering, environment variables, quiet mode, failure reporting, and working-directory behavior.
- Adds command documentation and updates command listings.
