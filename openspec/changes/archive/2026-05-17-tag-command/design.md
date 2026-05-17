## Context

`git-vmr` is a Rust CLI for operating on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing aggregate commands resolve an effective working directory once, discover the VMR root from that directory, scan immediate child directories, skip non-Git children for inventory-style operations, and shell out to Git for repository-local behavior.

`git vmr branch` already answers a local branch inventory question across child repositories. The new tag command should answer the equivalent tag inventory question without introducing tag mutation or verification behavior.

## Goals / Non-Goals

**Goals:**
- Add `git vmr tag` as a read-only command for listing local tags across immediate child Git repositories.
- Use tag-centric output so shared tags collapse to one line and partial tags call out the repositories where they exist.
- Keep output deterministic by sorting repository names and tag names.
- Reuse the existing VMR root discovery model and error behavior for broken Git repositories.

**Non-Goals:**
- Creating, deleting, replacing, annotating, signing, or verifying tags.
- Filtering tags with patterns or options such as `--contains`, `--points-at`, `--merged`, or `--format`.
- Listing remote refs or non-tag refs.
- Recursively discovering nested repositories.
- Adding configuration for repository inclusion or exclusion.

## Decisions

### Decision: List local tag refs only

The command should collect tags from `refs/tags` in each child repository. This keeps `git vmr tag` aligned with plain `git tag`, where no arguments list local tags by default.

Alternative considered: include richer tag metadata such as the target object, annotation subject, or tagger date. That would make the command more diagnostic, but it also pulls the first version away from simple tag inventory and introduces formatting questions better left to future options.

### Decision: Tag-centric rendering

Results should be grouped by tag name rather than rendered repo-by-repo. A tag line should include repository names only when the tag is absent from at least one discovered repository. When a tag exists in every discovered repository, the line should omit the repository list.

This makes the common aligned release case compact:

```text
v1.0.0
v1.1.0
```

and makes partial tag presence explicit:

```text
v1.0.0
v1.1.0 (backend, frontend)
v2.0.0-rc1 (tools)
```

Alternative considered: repo-centric output that repeats `git tag` under each repository. That is familiar, but it makes the user visually compare tag lists by hand, which is the work the VMR command should remove.

### Decision: No active marker or HEAD state rendering

Tags are refs, not checked-out state. Unlike branch listing, there is no Git marker equivalent to indicate an active tag, and detached HEAD state does not change which local tag refs exist. The tag command should therefore render plain tag lines only.

Alternative considered: mark tags that point at the current `HEAD` in at least one repository. That could be useful for release checks, but it is not plain `git tag` list behavior and would blur tag inventory with state inspection.

### Decision: Shell out to Git with focused commands

The implementation should shell out to Git per repository using `--no-optional-locks`, consistent with existing command modules. Tag refs can be collected with `git for-each-ref --format=%(refname:short) refs/tags`.

Alternative considered: use a Rust Git library. The project currently shells out for Git semantics, and this command needs only a stable, narrow Git CLI query.

## Risks / Trade-offs

- Lightweight and annotated tags both appear as names only -> Keep v1 as inventory; future options can expose metadata if needed.
- A partial tag line does not explain why repositories differ -> The command highlights the mismatch; users can inspect individual repositories for cause.
- Git command behavior varies around corrupted repositories -> Follow `branch` and fail the whole command when any child that appears to be a Git repository cannot be queried.
- Many repositories mean many Git subprocesses -> The command is read-only and parallelizable; collect per-repo tag info in parallel and render sequentially for deterministic output.
