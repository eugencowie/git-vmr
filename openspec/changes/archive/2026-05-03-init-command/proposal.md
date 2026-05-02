## Why

git-vmr needs a foundational `init` command to bootstrap a virtual monorepo workspace. Without it, there is no way to create the `.gitvmr/` directory and config file that all other commands will depend on.

## What Changes

- Add `git vmr init` command that creates a `.gitvmr/` directory with a config file in the current working directory
- The command is idempotent — safe to re-run without overwriting existing config
- No git repository is required

## Capabilities

### New Capabilities
- `init-command`: Bootstraps a virtual monorepo by creating the `.gitvmr/config` file with default settings

### Modified Capabilities
<!-- No existing capabilities to modify -->

## Impact

- **Entry point**: The current entry point is replaced with CLI infrastructure
- **Dependencies**: New dependencies added for CLI parsing, config serialization, and error handling
