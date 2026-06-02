## Why

There is no way to run `git-vmr` commands against a directory other than the current working directory. Users must `cd` into the target directory first. A `-C` flag (matching `git -C`) would allow scripting and automation without changing the shell's working directory.

## What Changes

- Add a global `-C <path>` flag to the `git-vmr` CLI that sets the working directory for all subcommands
- If the path does not exist or is not accessible, an error is returned before any subcommand runs

## Capabilities

### New Capabilities
- `working-dir-argument`: Global `-C <path>` flag that sets the working directory

### Modified Capabilities
- `init-command`: Init will use the directory from `-C` when provided

## Impact

- CLI argument parsing
- Error handling for cases where the provided directory does not exist or is not accessible
