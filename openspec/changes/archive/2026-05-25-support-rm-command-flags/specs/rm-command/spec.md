## ADDED Requirements

### Requirement: Rm supports force removal
The `git vmr rm` command SHALL accept `-f` and `--force` and SHALL pass force removal to each underlying `git rm` invocation after VMR path routing succeeds.

#### Scenario: Remove modified tracked file with short force flag
- **WHEN** user runs `git vmr rm -f backend/src/main.rs` and `src/main.rs` has working tree modifications in the `backend` repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

#### Scenario: Remove modified tracked file with long force flag
- **WHEN** user runs `git vmr rm --force backend/src/main.rs` and `src/main.rs` has working tree modifications in the `backend` repository
- **THEN** the command SHALL remove `src/main.rs` from the `backend` working tree and stage the deletion in `backend`

### Requirement: Rm supports dry-run previews
The `git vmr rm` command SHALL accept `-n` and `--dry-run` and SHALL pass dry-run mode to each underlying `git rm` invocation after VMR path routing succeeds. In dry-run mode, the command SHALL report Git's preview output and SHALL NOT remove files from child repository working trees or indexes.

#### Scenario: Preview file removal with short dry-run flag
- **WHEN** user runs `git vmr rm -n backend/src/main.rs`
- **THEN** the command SHALL report that `src/main.rs` would be removed from `backend`
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree
- **AND** the deletion SHALL NOT be staged in `backend`

#### Scenario: Preview file removal with long dry-run flag
- **WHEN** user runs `git vmr rm --dry-run backend/src/main.rs`
- **THEN** the command SHALL report that `src/main.rs` would be removed from `backend`
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree
- **AND** the deletion SHALL NOT be staged in `backend`

#### Scenario: Preview VMR root removal still requires recursive flag
- **WHEN** user runs `git vmr rm -n .` from the VMR root
- **THEN** the command SHALL fail before invoking any `git rm` operation

#### Scenario: Preview recursive VMR root removal
- **WHEN** user runs `git vmr rm -n -r .` from the VMR root containing `backend` and `frontend` child Git repositories
- **THEN** the command SHALL report removals that would occur in `backend` and `frontend`
- **AND** no deletion SHALL be staged in either child repository
- **AND** no tracked file SHALL be removed from either child repository working tree

### Requirement: Rm supports cached index-only removal
The `git vmr rm` command SHALL accept `--cached` and SHALL pass cached mode to each underlying `git rm` invocation after VMR path routing succeeds. In cached mode, the command SHALL remove routed paths from child repository indexes while leaving working tree files in place.

#### Scenario: Remove tracked file from index only
- **WHEN** user runs `git vmr rm --cached backend/src/main.rs`
- **THEN** the command SHALL stage removal of `src/main.rs` from the `backend` index
- **AND** `src/main.rs` SHALL remain present in the `backend` working tree

#### Scenario: Remove tracked directory from index only with recursive flag
- **WHEN** user runs `git vmr rm --cached -r backend/src`
- **THEN** the command SHALL stage removal of tracked paths under `src` from the `backend` index
- **AND** the files under `src` SHALL remain present in the `backend` working tree

### Requirement: Rm flags preserve VMR path validation
The `git vmr rm` command SHALL validate VMR ownership for all explicit pathspecs before invoking any `git rm` operation, even when `-f`, `--force`, `-n`, `--dry-run`, or `--cached` is present.

#### Scenario: Force does not allow VMR metadata path
- **WHEN** user runs `git vmr rm --force .gitvmr/config` from the VMR root
- **THEN** the command SHALL fail without removing files from any child repository

#### Scenario: Dry-run does not allow explicit non-Git child path
- **WHEN** user runs `git vmr rm --dry-run docs/readme.md` and `docs` is an immediate child directory that is not a Git repository
- **THEN** the command SHALL fail without invoking any `git rm` operation

#### Scenario: Cached mode does not allow path outside VMR
- **WHEN** user runs `git vmr rm --cached ../outside.txt` and the resolved path is outside the VMR root
- **THEN** the command SHALL fail without removing files from any child repository index
