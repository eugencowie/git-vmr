## Purpose
Define user-level global configuration file resolution, loading behavior, and CLI context access.

## Requirements

### Requirement: Resolve user-level global config path
The CLI SHALL resolve the global configuration file as `git-vmr/config.toml` under the user-level platform configuration directory.

#### Scenario: Platform config directory is available
- **WHEN** the platform configuration directory resolves to `/home/user/.config`
- **THEN** the global configuration path SHALL be `/home/user/.config/git-vmr/config.toml`

#### Scenario: Config directory override is provided
- **WHEN** `GIT_VMR_CONFIG_DIR` is set to `/tmp/git-vmr-config`
- **THEN** the global configuration path SHALL be `/tmp/git-vmr-config/git-vmr/config.toml`

### Requirement: Load missing global config as defaults
The CLI SHALL treat a missing global configuration file as default global configuration.

#### Scenario: Global config file does not exist
- **WHEN** the resolved global configuration path does not exist
- **THEN** global configuration loading SHALL succeed with default values

### Requirement: Reject malformed global config before running commands
The CLI SHALL fail before executing the requested subcommand when the resolved global configuration file exists but cannot be parsed.

#### Scenario: Global config contains invalid TOML
- **WHEN** user runs `git vmr status` and the resolved global configuration file contains invalid TOML
- **THEN** the command SHALL fail before VMR root discovery
- **AND** stderr SHALL contain a global configuration parse failure
- **AND** stdout SHALL be empty

#### Scenario: Global config contains invalid value types
- **WHEN** user runs `git vmr status` and the resolved global configuration file contains a value with the wrong type
- **THEN** the command SHALL fail before VMR root discovery
- **AND** stderr SHALL contain a global configuration parse failure
- **AND** stdout SHALL be empty

### Requirement: Keep repository config separate from global config
The global configuration file SHALL NOT replace or modify repository-local `.gitvmr/config` behavior.

#### Scenario: Init creates repository-local config
- **WHEN** user runs `git vmr init` in a directory without `.gitvmr/`
- **THEN** `.gitvmr/config` SHALL be created in the target directory
- **AND** the global configuration file SHALL NOT be created as part of init

#### Scenario: Existing repository-local config is preserved
- **WHEN** user runs `git vmr init` in a directory with an existing `.gitvmr/config`
- **THEN** `.gitvmr/config` SHALL NOT be overwritten
- **AND** global configuration loading SHALL NOT change repository-local config contents

### Requirement: Provide shared CLI context
The CLI SHALL dispatch commands with a shared context that exposes the invoked command name, effective working directory, and global configuration.

#### Scenario: Command runs with effective working directory
- **WHEN** user runs `git vmr -C /workspace/vmr status`
- **THEN** command dispatch SHALL receive `/workspace/vmr` as the effective working directory through the shared context

#### Scenario: Command renders invoked command name
- **WHEN** user invokes the binary as `git-vmr`
- **THEN** command dispatch SHALL receive `git vmr` as the display command name through the shared context
