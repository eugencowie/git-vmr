## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing aggregate commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository-local semantics, and report repository names when rendering aggregate output.

`git vmr fetch [<repository> [<refspec>...]]` and `git vmr pull [<repository> [<refspec>...]]` provide the closest implementation models: they accept an optional repository plus trailing refspecs, run child repository attempts in parallel, buffer per-repository Git output, and report aggregate results deterministically after every repository has been attempted. `git vmr push [<repository> [<refspec>...]]` should follow that model, while accounting for push's remote side effects: it can publish commits, reject non-fast-forward updates, trigger remote hooks, or require credentials independently per child repository.

## Goals / Non-Goals

**Goals:**

- Add `git vmr push [<repository> [<refspec>...]]` for pushing from every immediate child Git repository.
- Preserve Git's interpretation of the optional repository and refspec arguments in each child repository.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository push attempts in parallel.
- Report successful push output and failed push output with repository suffixes in deterministic repository-name order.
- Reuse existing global `-C <path>` and VMR root discovery behavior.
- Leave each local and remote repository in the state produced by its own `git push` attempt, including successful published refs and rejected pushes.

**Non-Goals:**

- Do not add push option flags such as `--force`, `--force-with-lease`, `--tags`, `--set-upstream`, `--delete`, `--atomic`, or generic trailing passthroughs in the initial command.
- Do not pre-resolve remote names, repository URLs, upstream branches, default push remotes, push.default behavior, authentication, remote hooks, or refspecs before invoking Git.
- Do not synchronize remote configuration, branch configuration, or push policy across child repositories.
- Do not recursively discover nested repositories or add configured repository inclusion/exclusion.
- Do not provide cross-repository rollback, transaction semantics, or remote cleanup after partial success.
- Do not provide live per-repository progress streaming in the initial implementation.

## Decisions

### Decision: Model push as an aggregate VMR command

The command should discover the VMR root, collect immediate child Git repositories, and invoke `git --no-optional-locks -C <repo> push [<repository> [<refspec>...]]` in each child repository.

Alternative considered: treat push like a plain Git passthrough from the effective working directory. That would publish only one repository and would not solve the VMR repetition problem.

### Decision: Keep the initial argument surface to repository and refspecs

The parser should support one optional repository argument followed by zero or more refspec arguments. The first positional argument is always the Git push repository argument, meaning `git vmr push main` pushes to a repository or remote named `main`; it is not interpreted as a refspec for the default remote.

Alternative considered: collect arbitrary trailing arguments and forward them to Git. That would support more Git syntax immediately, but it weakens the documented contract, complicates help text and tests, and makes later option-specific behavior harder to specify.

### Decision: Delegate push semantics and validation to Git

The command should not validate whether a child repository has a matching remote, configured upstream branch, default push remote, clean working tree, reachable repository URL, valid refspec, or acceptable remote policy. Each child repository may have different remotes, branches, push.default configuration, credentials, and hooks, so Git should decide success or failure per repository.

Alternative considered: preflight repository state before pushing. That could produce clearer failures for common cases, but it would duplicate Git semantics, risk diverging from local configuration, and still need to handle races between preflight and push.

### Decision: Run pushes in parallel with buffered reporting

Push attempts should run in parallel, with each worker capturing Git stdout and stderr. After all workers finish, results should be rendered in deterministic repository-name order.

Alternative considered: stream each child Git process directly to the terminal. That would preserve live progress, but parallel output would interleave and become hard to attribute to repositories. Buffered reporting is consistent with the existing aggregate command pattern.

### Decision: Report first useful Git line per repository

On success, if Git emits output, the command should print the first non-empty line from Git stderr, falling back to stdout, followed by the repository name. If Git emits no output, that repository contributes no success line. On failure, stderr should contain the first non-empty line from Git stderr, falling back to stdout, followed by the repository name.

Alternative considered: preserve full multi-line push output for every repository. That keeps more detail, but it can become noisy in large VMRs and is harder to scan after parallel execution. A concise first-line report matches existing command behavior while leaving detailed investigation to running Git directly in the affected repository.

### Decision: Best-effort means no cross-repository transaction

If push succeeds in one repository and fails in another, the successful push remains published and the failed repository remains unchanged or rejected according to Git. The command should return a non-zero status when any repository fails, after attempting every discovered child Git repository.

Alternative considered: stop after the first failure or attempt rollback. Stopping early would leave users without a complete view of which repositories can publish successfully. Rollback would require remote-specific cleanup semantics and cannot reliably undo already accepted pushes.

## Risks / Trade-offs

- [Risk] Captured output hides live push progress and can make interactive credential prompts less usable -> Mitigation: keep v1 deterministic and document richer passthrough or streaming behavior as a possible future extension.
- [Risk] Parallel pushes can trigger multiple credential prompts or remote hook executions at once -> Mitigation: delegate authentication and hook behavior to Git and report every failed repository.
- [Risk] Partial success can publish some repositories while others fail -> Mitigation: make best-effort semantics explicit and return a non-zero status when any push fails.
- [Risk] The same repository argument may mean different remotes or URLs per child repository -> Mitigation: delegate to Git and report per-repository failures.
- [Risk] First-line output can omit details from multi-line push reports -> Mitigation: keep default output concise and leave detailed investigation to running Git directly in the affected repository.
- [Risk] Parallel network operations can stress shared remotes or credentials helpers -> Mitigation: match existing aggregate parallelism and keep repository attempts independent.
