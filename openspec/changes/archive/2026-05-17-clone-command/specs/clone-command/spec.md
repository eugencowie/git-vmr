## ADDED Requirements

### Requirement: Clone delegates to Git

The `git vmr clone <repo> [<dir>]` command SHALL invoke Git clone for the provided repository and optional destination directory without applying VMR root discovery or child repository aggregation.

#### Scenario: Clone repository into inferred directory

- **WHEN** user runs `git vmr clone /repos/project.git` from `/workspace`
- **THEN** the command SHALL invoke Git clone from `/workspace`
- **AND** Git SHALL create the clone using its normal inferred directory behavior

#### Scenario: Clone repository into explicit directory

- **WHEN** user runs `git vmr clone /repos/project.git project-copy` from `/workspace`
- **THEN** the command SHALL invoke Git clone from `/workspace`
- **AND** Git SHALL clone `/repos/project.git` into `/workspace/project-copy`

### Requirement: Clone honors global working directory

The `git vmr clone <repo> [<dir>]` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as Git's clone working directory.

#### Scenario: Clone destination is relative to -C

- **WHEN** user runs `git vmr -C /workspace/base clone /repos/project.git project-copy`
- **THEN** the command SHALL invoke Git clone from `/workspace/base`
- **AND** Git SHALL clone `/repos/project.git` into `/workspace/base/project-copy`

### Requirement: Clone does not require VMR metadata

The `git vmr clone <repo> [<dir>]` command SHALL work from directories that do not contain `.gitvmr/` and SHALL NOT create `.gitvmr/` metadata as part of cloning.

#### Scenario: Clone outside a virtual monorepo

- **WHEN** user runs `git vmr clone /repos/project.git project-copy` from a directory with no `.gitvmr/` ancestor
- **THEN** the command SHALL attempt the Git clone
- **AND** the command SHALL NOT fail because VMR metadata is absent
- **AND** the command SHALL NOT create `.gitvmr/`

### Requirement: Clone preserves Git process behavior

The `git vmr clone <repo> [<dir>]` command SHALL preserve Git clone's stdout, stderr, interactive prompts, progress output, and exit status.

#### Scenario: Git clone succeeds

- **WHEN** user runs `git vmr clone /repos/project.git project-copy`
- **AND** Git clone exits successfully
- **THEN** `git vmr clone` SHALL exit successfully
- **AND** Git's clone output SHALL be visible to the user

#### Scenario: Git clone fails

- **WHEN** user runs `git vmr clone /repos/missing.git project-copy`
- **AND** Git clone exits with a non-zero status
- **THEN** `git vmr clone` SHALL exit with a non-zero status
- **AND** Git's clone error output SHALL be visible to the user
