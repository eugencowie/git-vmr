## ADDED Requirements

### Requirement: Tag command lists local tags across child repositories
The `git vmr tag` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and list local tag refs from each immediate child Git repository.

#### Scenario: Multiple child repositories with local tags
- **WHEN** a VMR contains child Git repositories `backend` and `frontend` and both have local tag `v1.0.0`
- **THEN** `git vmr tag` SHALL include a `v1.0.0` tag line

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **THEN** `git vmr tag` SHALL list tags from `backend` and SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **THEN** `git vmr tag` SHALL succeed with no output

#### Scenario: No local tags found
- **WHEN** a VMR contains child Git repositories but none has local tags
- **THEN** `git vmr tag` SHALL succeed with no output

### Requirement: Tag output is tag-centric
The `git vmr tag` command SHALL group output by local tag name across discovered child Git repositories and SHALL render tag lines in deterministic tag-name order.

#### Scenario: Shared tag omits repository list
- **WHEN** discovered repositories `backend` and `frontend` both have local tag `v1.0.0`
- **THEN** the output SHALL contain `v1.0.0` without a repository list

#### Scenario: Partial tag lists repositories
- **WHEN** discovered repositories are `backend` and `frontend` and only `frontend` has local tag `v1.1.0`
- **THEN** the output SHALL contain `v1.1.0 (frontend)`

#### Scenario: Partial tag lists multiple repositories
- **WHEN** discovered repositories are `backend`, `frontend`, and `tools` and only `backend` and `frontend` have local tag `v1.2.0`
- **THEN** the output SHALL contain `v1.2.0 (backend, frontend)`

### Requirement: Tag command uses existing VMR working directory behavior
The `git vmr tag` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr tag` from inside child repository `frontend` and a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root and list tags across all immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend tag` and `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root and list tags across its immediate child Git repositories

### Requirement: Tag command fails on child repository query errors
If a child directory appears to be a Git repository but tag information cannot be queried from it, the `git vmr tag` command SHALL fail with a non-zero exit status and an error message.

#### Scenario: Corrupted child Git repository
- **WHEN** a VMR contains a child directory with an invalid `.git` entry
- **THEN** `git vmr tag` SHALL fail and report that tag information could not be read

### Requirement: Tag command is list-only
The `git vmr tag` command SHALL expose only tag listing behavior in this change and SHALL reject unsupported tag creation, deletion, verification, signing, annotation, filtering, formatting, or pattern arguments during CLI argument validation.

#### Scenario: Tag creation is unsupported
- **WHEN** user runs `git vmr tag v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Tag deletion is unsupported
- **WHEN** user runs `git vmr tag -d v1.0.0`
- **THEN** the command SHALL fail during CLI argument validation

#### Scenario: Tag filtering is unsupported
- **WHEN** user runs `git vmr tag --contains HEAD`
- **THEN** the command SHALL fail during CLI argument validation
