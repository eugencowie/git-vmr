## ADDED Requirements

### Requirement: Init creates .gitvmr/config in current directory
The `git vmr init` command SHALL create a `.gitvmr/` directory and a `.gitvmr/config` file in the current working directory. The config file SHALL contain TOML with a `[core]` section and `version = 0`.

#### Scenario: Fresh init in empty directory
- **WHEN** user runs `git vmr init` in a directory without `.gitvmr/`
- **THEN** `.gitvmr/config` is created with content `[core]\nversion = 0`

#### Scenario: Init when .gitvmr directory exists but no config file
- **WHEN** user runs `git vmr init` and `.gitvmr/` directory exists but `.gitvmr/config` does not
- **THEN** `.gitvmr/config` is created with content `[core]\nversion = 0`

### Requirement: Init is idempotent
The `git vmr init` command SHALL silently succeed if `.gitvmr/config` already exists. It SHALL NOT overwrite or modify the existing config file.

#### Scenario: Re-running init with existing config
- **WHEN** user runs `git vmr init` and `.gitvmr/config` already exists
- **THEN** the command exits successfully without modifying the file

### Requirement: Init does not require a git repository
The `git vmr init` command SHALL NOT require the current directory to be inside a git repository. It SHALL work in any directory.

#### Scenario: Init outside a git repo
- **WHEN** user runs `git vmr init` in a directory that is not a git repository
- **THEN** `.gitvmr/config` is created successfully

### Requirement: CLI uses clap with subcommand pattern
The CLI SHALL use clap (derive) with a subcommand pattern. The binary SHALL be named `git-vmr` so that `git vmr <subcommand>` dispatches to it.

#### Scenario: Running git vmr init
- **WHEN** user runs `git vmr init`
- **THEN** the init subcommand is dispatched and executed
