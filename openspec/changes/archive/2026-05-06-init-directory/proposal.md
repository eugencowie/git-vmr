## Why

Users can initialize the current working directory or an existing `-C` working directory, but there is no direct way to initialize a named project directory in one command. Supporting an optional `init` directory argument makes `git vmr init <directory>` useful for project creation flows while preserving `-C` as an existing-directory execution context.

## What Changes

- Add an optional positional directory argument to `git vmr init`.
- When the argument is omitted, keep the existing behavior: initialize `.gitvmr` in the effective working directory.
- When the argument is provided and does not exist, create the directory and initialize `.gitvmr` inside it.
- When the argument is provided and exists as a directory, initialize or reinitialize `.gitvmr` inside it.
- Preserve global `-C <path>` semantics: `-C` still must refer to an existing directory, and relative init directory arguments are resolved from that effective working directory.
- Return an error when the init directory argument resolves to an existing non-directory path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `init-command`: Add optional target directory behavior for initialization.

## Impact

- CLI parsing for the `init` subcommand.
- Init command target path resolution and directory creation behavior.
- Unit tests for parsing, target resolution, missing target directory creation, `-C` interaction, idempotency, and non-directory errors.
