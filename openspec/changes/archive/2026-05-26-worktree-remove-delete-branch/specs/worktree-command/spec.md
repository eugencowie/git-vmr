## ADDED Requirements

### Requirement: Worktree remove deletes checked-out branches when requested
The `git vmr worktree remove -d <worktree>` and `git vmr worktree remove --delete <worktree>` commands SHALL attempt to safely delete the local branch that was checked out in each child worktree after that child worktree has been removed successfully. Branch deletion SHALL be delegated to Git using safe branch deletion semantics.

#### Scenario: Delete branch after successful child worktree removal
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** linked child worktrees exist at `../wt/backend` and `../wt/frontend`
- **AND** both child worktrees are checked out on local branch `wt`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL remove `../wt/backend` and `../wt/frontend`
- **AND** the command SHALL attempt to delete local branch `wt` in `backend`
- **AND** the command SHALL attempt to delete local branch `wt` in `frontend`
- **AND** the command SHALL exit successfully when both worktree removals and both safe branch deletions succeed

#### Scenario: Short delete flag is accepted
- **WHEN** linked child worktree `../wt/backend` is checked out on local branch `wt`
- **AND** user runs `git vmr worktree remove -d ../wt`
- **THEN** the command SHALL remove `../wt/backend`
- **AND** the command SHALL attempt to delete local branch `wt` in `backend`

#### Scenario: Delete uses actual child worktree branch
- **WHEN** linked child worktree `../wt/backend` is checked out on local branch `feature/auth`
- **AND** the aggregate worktree path basename is `wt`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL attempt to delete local branch `feature/auth` in `backend`
- **AND** the command SHALL NOT infer branch name `wt` from the aggregate worktree path

#### Scenario: Detached child worktree has no branch deletion
- **WHEN** linked child worktree `../wt/backend` is in detached HEAD state
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL remove `../wt/backend`
- **AND** the command SHALL NOT attempt to delete a local branch in `backend`

#### Scenario: Failed child worktree removal prevents branch deletion for that repository
- **WHEN** linked child worktree `../wt/backend` is checked out on local branch `wt`
- **AND** Git refuses to remove `../wt/backend`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL exit with a non-zero status
- **AND** the command SHALL NOT attempt to delete local branch `wt` in `backend`
- **AND** `../wt/backend` SHALL remain in place

#### Scenario: Partial removal deletes branches for successful repositories
- **WHEN** linked child worktree removal succeeds for `frontend`
- **AND** linked child worktree removal fails for `backend`
- **AND** both child worktrees were checked out on local branch `wt`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL attempt to delete local branch `wt` in `frontend`
- **AND** the command SHALL NOT attempt to delete local branch `wt` in `backend`
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Branch deletion failure is reported after worktree removal
- **WHEN** linked child worktree `../wt/backend` is checked out on local branch `wt`
- **AND** Git successfully removes `../wt/backend`
- **AND** Git refuses to safely delete local branch `wt` in `backend`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git branch deletion failure for `backend`

#### Scenario: Force remains scoped to worktree removal
- **WHEN** linked child worktree `../wt/backend` requires `--force` for Git worktree removal
- **AND** local branch `wt` cannot be safely deleted with `git branch -d`
- **AND** user runs `git vmr worktree remove --force --delete ../wt`
- **THEN** the command SHALL pass one force flag to Git worktree removal for `backend`
- **AND** the command SHALL attempt safe branch deletion for local branch `wt` in `backend`
- **AND** the command SHALL NOT force-delete local branch `wt`
- **AND** the command SHALL exit with a non-zero status when Git refuses safe branch deletion

#### Scenario: Complete worktree removal cleans aggregate marker even when branch deletion fails
- **WHEN** linked child worktree removal succeeds for every child Git repository
- **AND** at least one requested safe branch deletion fails
- **AND** `../wt/.gitvmr` exists
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** `../wt/.gitvmr` SHALL no longer exist
- **AND** `../wt` SHALL no longer exist when it is otherwise empty
- **AND** the command SHALL exit with a non-zero status

#### Scenario: Delete branch failures are deterministic
- **WHEN** branch deletion fails in child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr worktree remove --delete ../wt`
- **THEN** stderr SHALL report the `alpha` branch deletion failure before the `zeta` branch deletion failure
