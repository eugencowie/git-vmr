## Context

`git vmr tag` currently has two supported modes: without a tag name it lists local tags across child repositories, and with a tag name it creates a lightweight tag at `HEAD` in every discovered child Git repository. The command explicitly rejects `-d`/`--delete`, so users cannot remove a coordinated local tag through the VMR-level command.

Branch deletion already establishes the aggregate mutating command pattern for this project: parse a deletion flag, discover child repositories once, run Git independently per repository, preserve Git's repository-local semantics, and render deterministic per-repository success or failure messages.

## Goals / Non-Goals

**Goals:**

- Add local tag deletion with `git vmr tag -d <tagname>` and `git vmr tag --delete <tagname>`.
- Preserve Git's deletion behavior by delegating to `git tag -d <tagname>` in each child repository.
- Keep deletion best-effort across repositories, with deterministic per-repository reporting.
- Preserve existing tag listing and lightweight tag creation behavior.

**Non-Goals:**

- No force tag replacement or retagging support.
- No annotated, signed, or message-backed tag support.
- No explicit target object support for tag creation.
- No tag verification, filtering, formatting, or pattern listing support.
- No multi-tag deletion in one invocation.
- No rollback of successful deletions when another repository fails.

## Decisions

### Decision: Add a delete flag to the existing tag command

The CLI should accept `-d`/`--delete` with exactly one tag name and dispatch that combination to a new deletion path. `git vmr tag` remains list mode, and `git vmr tag <tagname>` remains lightweight creation mode.

Alternative considered: add a separate command such as `git vmr tag delete <tagname>`. That would avoid overloading the tag command, but it would diverge from Git's own `git tag -d <tagname>` shape and from the existing branch deletion precedent.

### Decision: Keep deletion single-tag only

Although plain `git tag -d` accepts multiple tag names, this change should support one tag name per invocation. That matches the current `tag_name: Option<String>` shape and keeps aggregate reporting straightforward.

Alternative considered: support `git vmr tag -d <tagname>...` immediately. That would match more of Git's surface, but it introduces a cross-product of repositories and tag names in result reporting and makes partial success harder to scan.

### Decision: Delegate deletion semantics to Git

The implementation should run `git tag -d <tagname>` in each child repository and let Git decide whether the tag exists and can be deleted. The VMR command should not preflight tag existence or synthesize its own validation.

Alternative considered: query refs first and only invoke deletion for repositories that contain the tag. That would reduce expected failures when a tag is missing in some repositories, but it would hide Git's own behavior and make the command less consistent with branch deletion and tag creation.

### Decision: Reuse aggregate result reporting

Deletion should reuse the existing aggregate command result flow used by branch deletion and tag creation. Each repository attempt should capture stdout/stderr, successful deletion messages should be printed with repository suffixes, failures should be reported with repository suffixes, and any failure should make the overall command return non-zero after all attempts finish.

Alternative considered: print raw Git output directly from each worker. That would make output ordering nondeterministic under parallel execution and would lose the repository context users need in a VMR.

## Risks / Trade-offs

- [Risk] Some repositories may delete the tag while others fail because the tag is missing or the repository is invalid. -> Mitigation: make deletion explicitly best-effort and report every failed repository clearly.
- [Risk] Successful deletion output is noisier than tag creation. -> Mitigation: preserve Git's deletion feedback with repository suffixes so users can see which repositories changed.
- [Risk] Single-tag deletion is less complete than Git's multi-tag form. -> Mitigation: keep the first implementation aligned with existing VMR command patterns; multi-tag deletion can be added later with an explicit result-reporting design.
- [Risk] Parallel execution can scramble output if workers write directly. -> Mitigation: buffer results and use deterministic aggregate printing.
