## ADDED Requirements

### Requirement: Worktree add supports explicit branch creation
The `git vmr worktree add -b <new-branch> <path> [<commit-ish>]` command SHALL create each child repository worktree on the requested new branch by passing `-b <new-branch>` to Git. When `<commit-ish>` is provided, the command SHALL pass it to Git as the start point for the new branch. When `<commit-ish>` is omitted, Git SHALL use its default start point behavior. The command SHALL NOT infer a branch name from the aggregate target path when `-b <new-branch>` is provided.

#### Scenario: Explicit branch without commit-ish
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add -b feature/auth ../wt`
- **THEN** the command SHALL attempt to create a Git worktree for `backend` using new branch `feature/auth` at `../wt/backend`
- **AND** the command SHALL attempt to create a Git worktree for `frontend` using new branch `feature/auth` at `../wt/frontend`
- **AND** the command SHALL NOT create branch `wt` from the aggregate target path basename

#### Scenario: Explicit branch with commit-ish start point
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr worktree add -b feature/auth ../wt main`
- **THEN** the command SHALL attempt `git worktree add -b feature/auth` with explicit commit-ish `main` in both repositories
- **AND** the created child worktrees SHALL be checked out on branch `feature/auth` when Git worktree addition succeeds

#### Scenario: Explicit branch creation failure is reported per repository
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** branch `feature/auth` already exists in both repositories
- **AND** user runs `git vmr worktree add -b feature/auth ../wt`
- **THEN** the command SHALL attempt explicit branch worktree addition in both repositories
- **AND** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the Git branch creation failure for `backend` and `frontend`

#### Scenario: Branch flag requires branch name
- **WHEN** user runs `git vmr worktree add -b`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted

#### Scenario: Long branch alias is rejected
- **WHEN** user runs `git vmr worktree add --branch feature/auth ../wt`
- **THEN** command parsing SHALL fail
- **AND** no child repository worktree additions SHALL be attempted
