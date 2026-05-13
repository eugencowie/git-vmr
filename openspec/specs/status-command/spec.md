## Purpose
Define the `git vmr status` command behavior for aggregating and rendering status across child repositories in a virtual monorepo.

## Requirements

### Requirement: Status shows unified git status across all immediate child repositories
The `git vmr status` command SHALL discover all immediate child directories of the VMR root, attempt to open each as a git repository, silently skip non-git directories, and collect the status of each git repository in parallel using rayon.

#### Scenario: Multiple repos with changes
- **WHEN** user runs `git vmr status` in a VMR root containing `backend/`, `frontend/`, and `docs/` where `docs/` is not a git repo
- **THEN** the command SHALL collect status from `backend/` and `frontend/` only, silently skipping `docs/`

#### Scenario: No child repositories found
- **WHEN** user runs `git vmr status` in a VMR root with no immediate child git repositories
- **THEN** the command SHALL succeed with no output

### Requirement: Status output is grouped by branch name
The output SHALL group repositories by their checked-out branch name. The committed branch group containing the most repositories SHALL omit the repository list from the branch header. When more than one committed branch group is tied for the most repositories, all tied branch groups SHALL list the repos in parentheses.

#### Scenario: All repos on the same branch
- **WHEN** `backend` and `frontend` are both on branch `main`
- **THEN** the output SHALL show `On branch main` without listing repo names

#### Scenario: One branch contains the most repos
- **WHEN** `frontend`, `backend`, and `docs` are on branch `develop`, `shared` and `common` are on branch `new-feature`, and `tools` is on branch `bug-fix`
- **THEN** the output SHALL show `On branch develop` without listing repo names
- **AND** the output SHALL show separate sections: `On branch new-feature (shared, common)` and `On branch bug-fix (tools)`

#### Scenario: Different branches are tied for the most repos
- **WHEN** `backend` is on `main` and `frontend` is on `feature/auth`
- **THEN** the output SHALL show separate sections: `On branch main (backend)` and `On branch feature/auth (frontend)`

### Requirement: Detached HEAD repos get individual sections
Repositories in detached HEAD state SHALL each appear in their own section with the header `HEAD detached at <short-hash> (<repo-name>)`. When color is enabled, the repository list in the header MAY be styled to match other repository-list annotations.

#### Scenario: Repo in detached HEAD state
- **WHEN** `tools` repo is in detached HEAD state at commit `a1b2c3d`
- **THEN** the output SHALL show a section with `HEAD detached at a1b2c3d (tools)`

### Requirement: Repos with no commits show initial state
Repositories with no commits yet SHALL show `On branch <name> (<repo-name>)` followed by `No commits yet`

#### Scenario: Newly initialized repo with no commits
- **WHEN** `new-repo` has been initialized with `git init` but has no commits and is on branch `master`
- **THEN** the output SHALL show `On branch master (new-repo)` followed by `No commits yet.`

### Requirement: File paths are relative to the working directory
All file paths in the status output SHALL be relative to the resolved working directory (from `-C` or cwd). Paths within the repo the user is inside SHALL have no prefix. Paths in sibling repos SHALL be prefixed with the repo name relative to the working directory.

#### Scenario: Running from VMR root
- **WHEN** user runs `git vmr status` from the VMR root and `backend/src/main.rs` has changes
- **THEN** the path SHALL appear as `backend/src/main.rs`

#### Scenario: Running from inside a child repo
- **WHEN** user runs `git vmr status` from within the `frontend/` directory and `backend/src/main.rs` has changes
- **THEN** the path SHALL appear as `../backend/src/main.rs`

### Requirement: Status command fails on any repo error
If opening or reading status from any git repository fails (excluding non-git directories which are skipped), the command SHALL fail with an error.

#### Scenario: Corrupted git repository
- **WHEN** one of the child directories contains a corrupted `.git/` directory
- **THEN** the command SHALL exit with a non-zero status and print an error message

### Requirement: Status collection runs in parallel
The command SHALL collect status from all discovered git repositories in parallel using rayon, then render the combined output sequentially in deterministic order.

#### Scenario: Many repos in the VMR
- **WHEN** a VMR root contains 10 git repositories
- **THEN** status collection SHALL run across all repos concurrently via rayon, and output SHALL be rendered in sorted order by repo name

### Requirement: File entries are color-coded
File entry lines SHALL be colored using `anstyle`: green for staged changes, red for unstaged changes, and red for untracked files. Repository-list annotations in headers MAY be styled. Section headers and status summary lines SHALL NOT be colored. Colors SHALL be automatically disabled when output is not a tty (e.g., piped to another command).

#### Scenario: Staged changes shown in green
- **WHEN** `backend/src/main.rs` has a staged modification
- **THEN** the file entry line SHALL be rendered in green

#### Scenario: Unstaged changes shown in red
- **WHEN** `frontend/src/app.rs` has an unstaged modification
- **THEN** the file entry line SHALL be rendered in red

#### Scenario: Untracked files shown in red
- **WHEN** `libs/new-file.txt` is untracked
- **THEN** the file entry line SHALL be rendered in red

#### Scenario: Colors disabled when piped
- **WHEN** output is piped to another command (stdout is not a tty)
- **THEN** all output SHALL be rendered without color codes
