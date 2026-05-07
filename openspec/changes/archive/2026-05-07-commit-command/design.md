## Context

`git-vmr` is a Rust CLI for operating on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing commands resolve an effective working directory once, discover the VMR root from that directory, and then either route explicit path operands into owning child repositories or scan immediate child repositories for aggregate operations.

`git vmr commit` is an aggregate mutating command. It should align with `status` and `branch` by scanning immediate child repositories, skipping non-Git child directories, producing deterministic output, and using Git's own porcelain behavior for repository-local commits.

## Goals / Non-Goals

**Goals:**
- Add `git vmr commit -m <message>` and `git vmr commit --message <message>` for committing staged changes across child Git repositories.
- Attempt commits in every immediate child Git repository that has staged changes.
- Keep output concise by printing only the first Git commit output line for successes, suffixed with the child repository name.
- Aggregate failures after attempting every eligible repository and report the first non-empty Git error line suffixed with the child repository name.
- Preserve existing `-C` working directory and VMR root discovery behavior.

**Non-Goals:**
- No `--allow-empty` support.
- No `-a` or `--all` support.
- No pathspec commit support.
- No editor-based commit message entry.
- No transactional rollback across repositories.

## Decisions

1. Treat `commit` as an aggregate command over child repositories with staged changes.

   Rationale: The command completes the current VMR edit flow: `add` stages routed paths, `status` shows the aggregate state, and `commit` commits the already-staged changes. Restricting v1 to staged changes avoids the surprising semantics of `git commit <pathspec>` and keeps ownership routing out of this command.

   Alternative considered: Accept pathspecs and route them like `add`, `restore`, and `rm`. That would copy one of Git commit's least obvious behaviors into the VMR layer and create ambiguity around whether unstaged working-tree content should be committed.

2. Skip clean repositories and non-Git child directories.

   Rationale: A normal VMR often has changes in only one child repository. Failing every clean sibling with Git's `nothing to commit` output would make the aggregate command noisy and hard to use. This matches the aggregate read-command pattern of skipping non-Git children.

   Alternative considered: Invoke `git commit` in every child Git repository. That would satisfy "attempt every repo" literally, but it would turn clean repos into routine failures and make multi-repo commits harder to reason about.

3. Attempt every eligible repository and aggregate failures.

   Rationale: The user explicitly wants one failing repository not to prevent attempts in the remaining repositories. This mirrors the independent-attempt model used by branch creation, while acknowledging that repository commits are not transactional.

   Alternative considered: Stop on the first failure. That avoids partial progress after a failure but leaves later repositories unattempted and forces users into manual cleanup.

4. Use Git CLI output as the source of commit summaries.

   Rationale: The first line of `git commit` output already contains the branch, abbreviated commit id, root-commit marker when applicable, and subject. Suffixing that line with the repository name gives VMR context without reimplementing Git's formatting.

   Alternative considered: Build summaries by querying commit metadata after each commit. That would add more Git calls and risk drifting from Git's own user-facing output.

5. Run commits in deterministic repository-name order.

   Rationale: Commits are mutating operations whose output should be predictable. Sequential deterministic execution also avoids interleaving stdout and stderr from Git hooks or signing prompts.

   Alternative considered: Run commits in parallel. That could be faster for large VMRs, but commit hooks, signing, and credential prompts make concurrent mutation harder to present and debug.

## Risks / Trade-offs

- Partial commits can occur when some repositories succeed and others fail -> Document the command as a batch of normal Git commits, not an atomic transaction, and return non-zero with repo-qualified errors.
- Commit hooks or signing prompts may emit multi-line output -> Use the first non-empty output or error line for VMR summaries and keep full Git behavior delegated to the underlying command.
- Repositories can become clean between preflight and commit -> Treat Git's commit failure as a repo-qualified failure and continue attempting the remaining eligible repositories.
- Initial repositories may produce root-commit output that differs from normal commits -> Preserve Git's first output line so root-commit summaries remain Git-compatible.
