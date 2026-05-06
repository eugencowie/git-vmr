## Context

`git vmr branch` currently behaves as a read-only branch inventory command: it discovers the VMR root, scans immediate child Git repositories, collects local branch refs in parallel, and renders branch-centric output. The CLI has no positional argument for the `branch` subcommand.

The new behavior extends the same command with an optional branch-name argument. With no argument, the command remains an inventory view. With a branch name, it becomes a bulk helper for creating the same local branch in each immediate child Git repository.

## Goals / Non-Goals

**Goals:**

- Preserve existing list-mode behavior for `git vmr branch`.
- Add `git vmr branch <branch-name>` to create a local branch in every immediate child Git repository.
- Run branch creation independently across repositories so failures in one repository do not prevent attempts in others.
- Run repository creation attempts in parallel while keeping final error output deterministic.
- Report concise Git-like failure lines annotated with repository names.

**Non-Goals:**

- Checking out the newly created branch.
- Preflighting branch existence or HEAD validity before attempting creation.
- Rolling back successful branch creations when other repositories fail.
- Creating remote branches or configuring upstream tracking.
- Adding verbosity flags or success output.

## Decisions

### Decision: Use one optional positional argument to switch modes

`git vmr branch` should remain list mode, while `git vmr branch <branch-name>` should create branches. This keeps the command aligned with plain `git branch`, where no branch name lists branches and a branch name creates a local branch.

Alternative considered: add a separate subcommand or flag, such as `git vmr branch create <name>`. That would avoid overloading the command, but it would be less consistent with Git's existing branch command shape.

### Decision: Best-effort creation without preflight

Create mode should attempt `git branch <branch-name>` in every immediate child Git repository and let Git decide whether each repository can create the branch. The command should not first validate all repositories or stop after the first failure.

This makes the operation simple and useful when some repositories are in different states. It also avoids a misleading promise of atomicity: even with preflight, repository state can change between validation and mutation.

Alternative considered: preflight all repositories and fail before mutation if any repo cannot create the branch. That reduces partial changes in common cases, but it conflicts with the desired best-effort behavior and still cannot guarantee atomicity.

### Decision: Parallel fan-out with buffered reporting

Branch creation attempts should run in parallel, similar to current branch inventory collection. Each worker should capture its Git result instead of writing directly to stdout or stderr. After all workers finish, failures should be sorted by repository name and rendered together.

This keeps large VMRs responsive while preserving deterministic output that does not depend on scheduling order.

Alternative considered: run attempts sequentially. That would make ordering natural, but it gives up easy parallelism and is unnecessary if output is buffered.

### Decision: Concise first-line failure output

If a repository fails, the final report should print the first non-empty line of Git stderr followed by the repository name in parentheses:

```text
fatal: a branch named 'feature/x' already exists (backend)
fatal: not a valid object name: 'HEAD' (tools)
```

The command should exit non-zero if any repository failed. If every repository succeeds, it should print nothing and exit successfully, matching plain `git branch <name>` success behavior.

Alternative considered: preserve full multi-line Git stderr per repository. That provides more detail, but it makes the default bulk command noisy. A concise first-line report is enough for common failures and keeps output scannable.

## Risks / Trade-offs

- Partial branch creation is expected by design -> Report every failed repository clearly and leave successful repositories unchanged.
- Parallel execution can make output nondeterministic if workers write directly -> Capture all results and sort failures by repository name before printing.
- First-line error reporting can omit Git hints -> Keep default output concise; richer diagnostics can be considered later if a verbose mode is added.
- Invalid branch names will fail in every repository -> Let Git provide the error message per repository, preserving best-effort semantics.
