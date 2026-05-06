## Context

`git-vmr` is a Rust CLI for operating on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing commands resolve an effective working directory once, discover the VMR root from that directory, scan immediate child directories, skip non-Git children for aggregate read operations, and shell out to Git for porcelain-compatible behavior.

`git vmr status` already groups checked-out branch state by repository, but it is intentionally status-oriented. The new branch command should answer a different question: which local branches exist across the virtual monorepo?

## Goals / Non-Goals

**Goals:**
- Add `git vmr branch` as a read-only command for listing local branches across immediate child Git repositories.
- Use branch-centric output so shared branches collapse to one line and partial branches call out the repositories where they exist.
- Match Git's active branch marker convention by rendering `*` in the left column when a branch is checked out in at least one repository.
- Render detached HEAD repositories as separate lines with short commit hashes and repository names.
- Keep output deterministic by sorting repository names and branch names.
- Reuse the existing VMR root discovery model and error behavior for broken Git repositories.

**Non-Goals:**
- Listing remote branches or all refs.
- Creating, deleting, renaming, or checking out branches.
- Reporting ahead/behind, upstream tracking, or branch divergence.
- Recursively discovering nested repositories.
- Adding configuration for repository inclusion or exclusion.

## Decisions

### Decision: List local branch refs only

The command should collect branches from `refs/heads` in each child repository. This keeps `git vmr branch` aligned with plain `git branch`, where remote refs require explicit flags and unborn branches in newly initialized repositories are not listed as local branch refs.

Alternative considered: include the symbolic HEAD name for repositories with no commits. That would make initial repositories visible, but it would produce output that plain `git branch` does not show and blur the meaning of "local branch exists".

### Decision: Branch-centric rendering

Results should be grouped by branch name rather than rendered repo-by-repo. A branch line should include repository names only when the branch is absent from at least one discovered repository. When a branch exists in every discovered repository, the line should omit the repository list.

This makes the common aligned case compact:

```text
* main
  release/1.2
```

and makes partial branch presence explicit:

```text
* main
  feature/auth (frontend)
  release/1.2 (backend, frontend)
```

Alternative considered: repo-centric output that repeats `git branch` under each repository. That is familiar, but it makes the user visually diff branch lists by hand, which is the work the VMR command should remove.

### Decision: Active marker is branch-line scoped

A branch line should begin with `*` when that branch is checked out in at least one repository; otherwise it should begin with a space. Repository names remain governed only by branch presence, not by active state. This means a branch that exists everywhere but is active in only one repository still renders without a repository list.

Alternative considered: include active repository names whenever the checked-out set differs. That would make current-branch alignment more explicit, but it conflicts with the requested "only show repo name if the branch does not exist in all repos" rule and overlaps with `git vmr status`.

### Decision: Detached HEADs render as individual Git-like lines

Detached HEAD repositories should not be grouped with branch refs. Each detached repository should render a separate active line:

```text
* (HEAD detached at 9773cf0) (frontend)
```

This mirrors Git's detached branch display while preserving the repository name that gives the line meaning in a VMR.

### Decision: Shell out to Git with focused commands

The implementation should shell out to Git per repository using `--no-optional-locks`, consistent with existing command modules. Branch refs can be collected with `git for-each-ref --format=%(refname:short) refs/heads`, active branch can be detected with `git symbolic-ref --quiet --short HEAD`, and detached HEAD hashes can be read with `git rev-parse --short HEAD` when symbolic ref detection fails.

Alternative considered: use a Rust Git library. The project currently shells out for Git semantics, and this command needs only stable, narrow Git CLI queries.

## Risks / Trade-offs

- **Active marker can be ambiguous for branches that exist everywhere** -> Keep the requested output rule and rely on `git vmr status` for detailed current-branch grouping.
- **Unborn repositories may produce no branch output** -> Treat the command as local-ref inventory and add tests so the behavior is deliberate.
- **Git command behavior varies around corrupted repositories** -> Follow `status` and fail the whole command when any child that appears to be a Git repository cannot be queried.
- **Many repositories mean many Git subprocesses** -> The command is read-only and parallelizable; collect per-repo branch info in parallel and render sequentially for deterministic output.
