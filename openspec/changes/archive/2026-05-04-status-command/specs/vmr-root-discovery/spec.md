## ADDED Requirements

### Requirement: VMR root discovery via ancestor traversal
The system SHALL provide a function that, given a starting directory path, traverses upward through parent directories looking for a `.gitvmr/` directory. The function SHALL return the parent directory of the `.gitvmr/` directory as the VMR root.

#### Scenario: Starting directory contains .gitvmr
- **WHEN** the starting directory is `/home/user/work` and `/home/user/work/.gitvmr/` exists
- **THEN** the function SHALL return `/home/user/work` as the VMR root

#### Scenario: Ancestor directory contains .gitvmr
- **WHEN** the starting directory is `/home/user/work/project-a/src` and `/home/user/work/.gitvmr/` exists
- **THEN** the function SHALL return `/home/user/work` as the VMR root

#### Scenario: No .gitvmr found in any ancestor
- **WHEN** the starting directory is `/home/user/work` and no `.gitvmr/` exists in `/home/user/work` or any ancestor up to the filesystem root
- **THEN** the function SHALL return an error indicating no virtual monorepo was found

### Requirement: VMR root discovery uses the -C working directory as starting point
The status command SHALL use the resolved working directory (from `-C` flag or current directory) as the starting point for VMR root discovery.

#### Scenario: Using -C flag as starting point
- **WHEN** user runs `git vmr -C /home/user/work/project-a status` and `/home/user/work/.gitvmr/` exists
- **THEN** the VMR root SHALL be discovered starting from `/home/user/work/project-a`

#### Scenario: Using current directory as starting point
- **WHEN** user runs `git vmr status` with current directory `/home/user/work/project-a` and `/home/user/work/.gitvmr/` exists
- **THEN** the VMR root SHALL be discovered starting from `/home/user/work/project-a`
