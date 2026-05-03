## Purpose
Define how the CLI resolves the working directory used by all subcommands.

## Requirements

### Requirement: CLI accepts -C flag to set working directory
The `git-vmr` CLI SHALL accept a global `-C <path>` flag. When provided, the CLI SHALL canonicalize the path to an absolute path and use it as the working directory for all subcommands. When `-C` is not provided, the CLI SHALL use `std::env::current_dir()` as the working directory.

#### Scenario: Running with -C flag
- **WHEN** user runs `git-vmr -C /some/path init`
- **THEN** the init subcommand SHALL receive `/some/path` (canonicalized) as its working directory

#### Scenario: Running without -C flag
- **WHEN** user runs `git-vmr init`
- **THEN** the init subcommand SHALL receive the current working directory as its working directory

### Requirement: -C flag canonicalizes the path
The CLI SHALL canonicalize the `-C` path using `std::fs::canonicalize` before dispatching to any subcommand.

#### Scenario: -C with relative path
- **WHEN** user runs `git-vmr -C ../foo init` and `../foo` resolves to an existing directory
- **THEN** the subcommand SHALL receive the canonicalized absolute path

### Requirement: -C flag errors on invalid path
The CLI SHALL surface an error if the path provided to `-C` does not exist or is not accessible. The error SHALL be displayed before any subcommand logic executes.

#### Scenario: -C with non-existent directory
- **WHEN** user runs `git-vmr -C /nonexistent init`
- **THEN** the CLI SHALL exit with an error indicating the directory could not be found

#### Scenario: -C with a file instead of directory
- **WHEN** user runs `git-vmr -C /path/to/file init` where `file` is not a directory
- **THEN** the CLI SHALL exit with an error
