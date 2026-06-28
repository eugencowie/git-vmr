## MODIFIED Requirements

### Requirement: Worktree add infers aggregate branch when commit-ish is omitted
When `<commit-ish>` is omitted and `-b <new-branch>` is not provided, `git vmr worktree add <path>` SHALL derive an inferred aggregate branch name from the basename of the aggregate target path. For each child repository, the command SHALL check out the existing local branch with that inferred name when it exists, and SHALL create a new branch with that inferred name when it does not exist locally.

#### Scenario: Omitted commit-ish creates same branch in every child worktree when absent
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** neither child repository has a local branch named `wt`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL derive branch name `wt` from aggregate target path `../wt`
- **AND** `../wt/backend` SHALL be created on new branch `wt`
- **AND** `../wt/frontend` SHALL be created on new branch `wt`

#### Scenario: Omitted commit-ish checks out existing local inferred branch
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** both child repositories have a local branch named `wt`
- **AND** branch `wt` is not checked out in another worktree for either child repository
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL derive branch name `wt` from aggregate target path `../wt`
- **AND** `../wt/backend` SHALL be created by checking out local branch `wt`
- **AND** `../wt/frontend` SHALL be created by checking out local branch `wt`
- **AND** the command SHALL exit successfully when both Git worktree additions succeed

#### Scenario: Omitted commit-ish handles mixed existing and absent inferred branches
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** `backend` has a local branch named `wt`
- **AND** `frontend` does not have a local branch named `wt`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL check out existing local branch `wt` for `backend`
- **AND** the command SHALL create new local branch `wt` for `frontend`
- **AND** the command SHALL exit successfully when both Git worktree additions succeed

#### Scenario: Existing inferred branch checked out elsewhere fails through Git
- **WHEN** a VMR contains child Git repository `backend`
- **AND** local branch `wt` is already checked out in another worktree for `backend`
- **AND** user runs `git vmr worktree add ../wt`
- **THEN** the command SHALL attempt to check out existing local branch `wt` for `backend`
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL contain Git's checked-out branch failure for `backend`
