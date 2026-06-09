## Why

git-vmr needs a user-level configuration file so future runtime features can read settings that apply across virtual monorepos. Introducing the file path and loading behavior first will provide a small, stable foundation without changing repository-local `.gitvmr/config` semantics.

## What Changes

- Add a user-level global configuration file at the platform config directory under `git-vmr/config.toml`.
- Allow tests and controlled environments to override the config root with `GIT_VMR_CONFIG_DIR`.
- Load the global configuration during CLI startup and fail before command execution if the file exists but cannot be parsed.
- Treat a missing global configuration file as default configuration.
- Keep repository-local `.gitvmr/config` behavior unchanged for `git vmr init` and VMR metadata.
- Add shared CLI context plumbing so commands can access the invoked command name, effective working directory, and global configuration through a single object.

## Capabilities

### New Capabilities
- `global-config-file`: Defines user-level global configuration path resolution, default loading behavior, parse failure handling, and test isolation.

### Modified Capabilities

## Impact

- Affected code: CLI startup, command dispatch signatures, configuration types, and integration test helpers.
- Affected dependencies: Adds a platform directory resolution dependency for locating the user config directory.
- Affected behavior: malformed global config files will fail before subcommands run; missing global config files will not require users to create any new file.
