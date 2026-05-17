## Why

`git vmr clone <repo> [<dir>]` is listed alongside startup commands, but it is currently unsupported even though users naturally expect `git vmr` to cover the basic Git entrypoint for creating a new work area. Supporting a narrow passthrough keeps the command surface familiar without introducing VMR-specific clone behavior.

## What Changes

- Add `git vmr clone <repo> [<dir>]` as a simple passthrough to `git clone`.
- Apply the existing global `-C <path>` option before invoking Git so clone destinations resolve from the effective working directory.
- Preserve Git's clone behavior, stdout, stderr, prompts, progress output, and exit status.
- Do not discover or require a `.gitvmr/` root for clone.
- Update command documentation to mark clone as supported.

## Capabilities

### New Capabilities

- `clone-command`: Defines `git vmr clone <repo> [<dir>]` behavior as a narrow passthrough to `git clone` from the effective working directory.

### Modified Capabilities

None.

## Impact

- Affects CLI parsing and dispatch in `src/cli.rs`.
- Adds clone command execution code and integration tests.
- Updates README and generated command documentation for clone support.
- Does not add dependencies or change existing VMR-aware command behavior.
