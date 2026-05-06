## MODIFIED Requirements

### Requirement: Init creates .gitvmr/config in target directory
The `git vmr init` command SHALL create a `.gitvmr/` directory and a `.gitvmr/config` file in the target directory. When no directory argument is provided, the target directory SHALL be the current working directory. When a directory argument is provided, the target directory SHALL be the provided directory. The config file SHALL contain TOML with a `[core]` section and `version = 0`.

#### Scenario: Fresh init in empty current directory
- **WHEN** user runs `git vmr init` in a directory without `.gitvmr/`
- **THEN** `.gitvmr/config` is created in the current directory with content `[core]\nversion = 0`

#### Scenario: Init when .gitvmr directory exists but no config file
- **WHEN** user runs `git vmr init` and `.gitvmr/` directory exists but `.gitvmr/config` does not
- **THEN** `.gitvmr/config` is created with content `[core]\nversion = 0`

#### Scenario: Init with existing directory argument
- **WHEN** user runs `git vmr init project` and `project/` exists
- **THEN** `project/.gitvmr/config` is created with content `[core]\nversion = 0`

#### Scenario: Init with missing directory argument
- **WHEN** user runs `git vmr init project` and `project/` does not exist
- **THEN** `project/` is created
- **AND** `project/.gitvmr/config` is created with content `[core]\nversion = 0`

#### Scenario: Init directory argument is relative to -C
- **WHEN** user runs `git vmr -C base init project` and `base/` exists
- **THEN** `base/project/.gitvmr/config` is created with content `[core]\nversion = 0`

#### Scenario: Init directory argument is an existing file
- **WHEN** user runs `git vmr init file` and `file` exists as a file
- **THEN** the command exits with an error
- **AND** `file/.gitvmr/config` is not created
