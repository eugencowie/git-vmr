## MODIFIED Requirements

### Requirement: CLI uses clap with subcommand pattern
The CLI SHALL use clap (derive) with a subcommand pattern. The binary SHALL be named `git-vmr` so that `git vmr <subcommand>` dispatches to it. The CLI SHALL accept a global `-C <path>` flag that overrides the working directory for all subcommands. When `-C` is provided, the path SHALL be canonicalized before dispatching. When `-C` is not provided, the current working directory SHALL be used.

#### Scenario: Running git vmr init
- **WHEN** user runs `git vmr init`
- **THEN** the init subcommand is dispatched and executed using the current working directory

#### Scenario: Running git vmr -C /path init
- **WHEN** user runs `git vmr -C /path init` and `/path` exists
- **THEN** the init subcommand is dispatched and executed using the canonicalized `/path`
