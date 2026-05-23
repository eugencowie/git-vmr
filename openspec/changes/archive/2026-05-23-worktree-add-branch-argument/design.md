## Context

`git vmr worktree add <path> [<commit-ish>]` currently creates one linked child worktree per immediate child Git repository. When `<commit-ish>` is omitted, VMR derives a shared branch name from the aggregate target path and passes that name to Git with `-b` internally, so `git vmr worktree add ../wt` creates branch `wt` in each child repository.

The Git helper already has a branch argument path, but the CLI does not expose user-selected branch names. Users who want aggregate worktrees on a branch that does not match the target directory currently need follow-up Git operations in each child repository.

## Goals / Non-Goals

**Goals:**

- Support `git vmr worktree add -b <new-branch> <path> [<commit-ish>]`.
- Use the explicit branch name in every child repository instead of the inferred aggregate target basename.
- Preserve Git's own behavior for branch validation, branch existence failures, commit-ish resolution, start-point handling, and worktree safety checks.
- Preserve existing inferred-branch behavior when `-b` is omitted.

**Non-Goals:**

- Do not add a `--branch` alias.
- Do not add `-B`, `--track`, `--guess-remote`, `--detach`, or other `git worktree add` flags.
- Do not preflight branch names, branch existence, or commit-ish validity before invoking Git.
- Do not add rollback for partial aggregate worktree creation failures.

## Decisions

### Decision: Parse only short `-b` on `worktree add`

The nested `worktree add` command should accept `-b <new-branch>` as an optional flag and should not define a long alias.

Rationale: Git documents `git worktree add -b <new-branch>`, and the requested VMR behavior is intentionally limited to that flag. Keeping the accepted surface narrow avoids implying broader branch option support such as `-B` or `--track`.

Alternative considered: expose `--branch` for readability. That would be convenient but would diverge from Git's documented worktree option shape and expand the command surface beyond the requested change.

### Decision: Treat explicit `-b` as an override for inferred branch naming

When `-b <new-branch>` is present, the command should pass that branch name to every child repository worktree addition. The aggregate target path basename should only be used when both `-b` and `<commit-ish>` are omitted.

Rationale: explicit user input should win over inference. This preserves the current convenience path while allowing users to choose a branch name that differs from the destination directory.

Alternative considered: reject `-b` unless `<commit-ish>` is provided. Git allows `-b <new-branch> <path>` with `HEAD` as the default start point, so rejecting that form would be unnecessarily restrictive.

### Decision: Keep commit-ish handling delegated to Git

For `git vmr worktree add -b feature ../wt main`, each child repository should receive the equivalent of `git worktree add -b feature <target>/<repo-name> main`. Git should decide whether `main` is valid and whether branch creation can proceed in that repository.

Rationale: existing VMR fan-out commands avoid duplicating Git validation and preserve repository-local errors. This keeps behavior consistent when some child repositories have different refs or branch state.

Alternative considered: pre-check that `<commit-ish>` exists in every repository before creating any worktree. That could reduce partial changes in common cases, but it would duplicate Git resolution rules and conflict with the established best-effort model.

## Risks / Trade-offs

- [Risk] A requested branch may be created in some child repositories while other child repositories fail. -> Mitigation: preserve existing best-effort behavior and deterministic repository-suffixed failure reporting.
- [Risk] Users may expect unsupported related options such as `-B` after seeing `-b`. -> Mitigation: keep parser support and documentation explicit that only `-b` is added in this change.
- [Risk] Branch names that are valid in one repository state but not another can produce mixed results. -> Mitigation: delegate validation to Git per child repository and surface Git's own error text.
