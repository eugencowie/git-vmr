## ADDED Requirements

### Requirement: Worktree remove supports visible rm alias
The `git vmr worktree rm <worktree>` command SHALL be a visible alias for `git vmr worktree remove <worktree>`. The alias SHALL parse the same operands and options as `remove`, SHALL dispatch to the same removal behavior, and SHALL be advertised in `git vmr worktree` help output.

#### Scenario: Alias removes linked aggregate worktree
- **WHEN** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** user runs `git vmr worktree rm ../wt`
- **THEN** the command SHALL behave as `git vmr worktree remove ../wt`
- **AND** `../wt/backend` and `../wt/frontend` SHALL no longer exist when Git succeeds

#### Scenario: Alias accepts remove flags
- **WHEN** user runs `git vmr worktree rm --force -D ../wt`
- **THEN** command parsing SHALL produce the same remove command shape as `git vmr worktree remove --force -D ../wt`

#### Scenario: Alias is shown in help
- **WHEN** user runs `git vmr worktree --help`
- **THEN** the help output SHALL advertise `rm` as an alias for the remove subcommand
