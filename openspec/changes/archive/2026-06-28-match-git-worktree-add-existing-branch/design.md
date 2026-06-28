## Context

`git vmr worktree add <path>` creates an aggregate VMR worktree by creating one child Git worktree at `<path>/<repo-name>` per child repository. When `<commit-ish>` is omitted, VMR currently derives the aggregate branch name from `<path>` and passes it to Git as `-b <branch>`, so an existing local branch fails with Git's branch-creation error.

Native `git worktree add <path>` uses the path basename as shorthand: if the local branch exists, Git checks it out; otherwise Git creates a new branch with that name. VMR cannot simply omit `-b` because each child Git command sees `<path>/<repo-name>` and would infer branch names from child repository names.

## Goals / Non-Goals

**Goals:**

- Match native Git's existing-local-branch shorthand for omitted-`<commit-ish>` `worktree add`.
- Preserve VMR's aggregate branch inference from the aggregate target basename.
- Preserve explicit `-b <new-branch>` create-only behavior.
- Preserve explicit `<commit-ish>` delegation to Git.

**Non-Goals:**

- Do not add new CLI flags or aliases.
- Do not implement full remote-branch guessing parity for `git worktree add <path>`.
- Do not add rollback or preflight validation across every child repository.

## Decisions

### Decision: Split inferred branch handling from explicit branch creation

The command layer should treat omitted-`<commit-ish>` with no `-b` as an inferred branch mode, not as explicit branch creation. For each child repository, it should derive the aggregate branch name once from the aggregate target basename and then choose the Git invocation from that repository's local branch state.

Rationale: this preserves the shared aggregate branch name while avoiding Git's `-b` branch-exists failure when the branch already exists.

Alternative considered: keep mapping omitted `<commit-ish>` to `-b`. That preserves current tests, but it is the source of the mismatch with native Git.

### Decision: Check only local branch existence before choosing the Git form

When the inferred aggregate branch exists locally in a child repository, call `git worktree add <child-target> <branch>`. When it does not exist locally, call `git worktree add -b <branch> <child-target>`.

Rationale: this delegates checkout safety, checked-out branch failures, and success output to Git while avoiding accidental detached worktrees for non-branch refs with the same name.

Alternative considered: first try `git worktree add <child-target> <branch>` and retry with `-b` on failure. That can detach `HEAD` when a tag exists with the inferred name but no local branch, which differs from Git's one-argument branch-creation behavior.

Alternative considered: delegate omitted `<commit-ish>` directly to Git without `-b` or explicit branch. That would make Git infer branch names from `<child-target>` basenames, such as `backend` or `frontend`, instead of the aggregate branch name.

### Decision: Keep explicit modes unchanged

`worktree add -b <new-branch> <path> [<commit-ish>]` should continue to pass `-b` to Git. `worktree add <path> <commit-ish>` should continue to pass the explicit commit-ish directly to Git.

Rationale: users who ask for new branch creation should still get Git's branch-exists failure, and explicit commit-ish handling already matches Git delegation.

## Risks / Trade-offs

- [Risk] Remote-only branches with the inferred name are still not treated exactly like native Git shorthand. -> Mitigation: keep this change focused on local existing branches and leave remote guessing as a separate capability if users need it.
- [Risk] Branch state can differ across child repositories, producing mixed checkout/create behavior. -> Mitigation: keep existing best-effort execution and deterministic repository-suffixed reporting.
- [Risk] New local branch detection duplicates a small part of Git behavior. -> Mitigation: limit it to verifying `refs/heads/<branch>` and let Git handle all checkout and creation details.
