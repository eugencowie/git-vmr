## Context

`git vmr tag` currently behaves as a read-only tag inventory command: it discovers the VMR root, scans immediate child Git repositories, collects local tag refs in parallel, and renders tag-centric output. The CLI has no positional argument for the `tag` subcommand and explicitly rejects `git vmr tag <tag-name>`.

`git vmr branch` already uses an optional positional argument to switch from inventory mode to bulk branch creation. Tag creation should follow that precedent where Git semantics align: no argument lists refs, and one name creates a local ref in each child repository.

## Goals / Non-Goals

**Goals:**

- Preserve existing list-mode behavior for `git vmr tag`.
- Add `git vmr tag <tag-name>` to create a lightweight local tag at `HEAD` in every immediate child Git repository.
- Run tag creation independently across repositories so failures in one repository do not prevent attempts in others.
- Run repository tag creation attempts in parallel while keeping final error output deterministic.
- Report concise Git-like failure lines annotated with repository names.

**Non-Goals:**

- Creating annotated or signed tags.
- Creating tags that point at an explicit object or commit argument.
- Replacing existing tags with `--force`.
- Deleting or verifying tags.
- Filtering, formatting, pattern matching, or changing list-mode output.
- Rolling back successful tag creations when other repositories fail.
- Adding verbosity flags or success output.

## Decisions

### Decision: Use one optional positional argument to switch modes

`git vmr tag` should remain list mode, while `git vmr tag <tag-name>` should create lightweight tags. This keeps the command aligned with plain `git tag`, where no arguments list tags and one tag name creates a tag.

Alternative considered: add a separate flag or subcommand, such as `git vmr tag create <name>`. That would avoid overloading the command, but it would be less consistent with Git's existing tag command shape and the existing `git vmr branch <branch-name>` behavior.

### Decision: Create lightweight tags at HEAD only

Create mode should invoke `git tag <tag-name>` in each child repository and let Git resolve `HEAD`, validate the tag name, and reject invalid repository states. This deliberately avoids accepting an optional target object for now.

Alternative considered: support `git vmr tag <tag-name> <object>` immediately. That would match more of Git's surface, but it introduces cross-repository object-resolution questions and increases the chance that the same command creates tags pointing to unrelated or missing objects across repositories.

### Decision: Best-effort creation without preflight

Create mode should attempt tag creation in every immediate child Git repository and let Git decide whether each repository can create the tag. The command should not first validate all repositories or stop after the first failure.

This matches branch creation semantics and avoids implying atomicity. Even with preflight, repository state can change between validation and mutation, and tag creation across multiple repositories cannot be rolled back reliably without creating a different, more destructive workflow.

Alternative considered: preflight every repository and fail before mutation if any repo cannot create the tag. That reduces partial changes in common cases, but it conflicts with the useful best-effort behavior already established by branch creation.

### Decision: Parallel fan-out with buffered reporting

Tag creation attempts should run in parallel, similar to current tag inventory collection and branch creation. Each worker should capture its Git result instead of writing directly to stdout or stderr. After all workers finish, failures should be sorted by repository name and rendered together.

Alternative considered: run attempts sequentially. That would make ordering natural, but it gives up straightforward parallelism and is unnecessary when output can be buffered.

### Decision: Concise first-line failure output

If a repository fails, the final report should print the first non-empty line of Git stderr followed by the repository name in parentheses:

```text
fatal: tag 'v1.0.0' already exists (backend)
fatal: Failed to resolve 'HEAD' as a valid ref. (tools)
```

The command should exit non-zero if any repository failed. If every repository succeeds, it should print nothing and exit successfully, matching plain `git tag <name>` success behavior.

Alternative considered: preserve full multi-line Git stderr per repository. That provides more detail, but it makes the default bulk command noisy. A concise first-line report is enough for common failures and keeps output scannable.

## Risks / Trade-offs

- Partial tag creation is expected by design -> Report every failed repository clearly and leave successful repositories unchanged.
- Parallel execution can make output nondeterministic if workers write directly -> Capture all results and sort failures by repository name before printing.
- First-line error reporting can omit Git hints -> Keep default output concise; richer diagnostics can be considered later if a verbose mode is added.
- Tagging `HEAD` across repositories can mark different commits -> This is the intended VMR behavior for coordinated repository-local refs; explicit object targeting remains out of scope.
- Invalid tag names will fail in every repository -> Let Git provide the error message per repository, preserving best-effort semantics.
