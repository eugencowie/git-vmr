## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing aggregate commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository-local semantics, and report repository names when rendering aggregate output.

`git vmr fetch [<repository> [<refspec>...]]` already provides the closest implementation model: it accepts an optional repository plus trailing refspecs, runs child repository attempts in parallel, buffers per-repository Git output, and reports aggregate results deterministically after every repository has been attempted. `git vmr pull [<repository> [<refspec>...]]` should follow that model, while accounting for pull's stronger side effects: it can update working trees, create merge commits, rebase branches depending on Git configuration, or leave conflicts in individual child repositories.

## Goals / Non-Goals

**Goals:**

- Add `git vmr pull [<repository> [<refspec>...]]` for pulling in every immediate child Git repository.
- Preserve Git's interpretation of the optional repository and refspec arguments in each child repository.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository pull attempts in parallel.
- Report successful pull output and failed pull output with repository suffixes in deterministic repository-name order.
- Reuse existing global `-C <path>` and VMR root discovery behavior.
- Leave each child repository in the state produced by its own `git pull` attempt, including successful updates or conflict states.

**Non-Goals:**

- Do not add pull option flags such as `--rebase`, `--ff-only`, `--no-commit`, `--autostash`, `--all`, or generic trailing passthroughs in the initial command.
- Do not pre-resolve remote names, repository URLs, upstream branches, pull strategy configuration, or refspecs before invoking Git.
- Do not synchronize remote configuration, branch configuration, or pull strategy across child repositories.
- Do not recursively discover nested repositories or add configured repository inclusion/exclusion.
- Do not provide cross-repository rollback, transaction semantics, or conflict recovery.
- Do not provide live per-repository progress streaming in the initial implementation.

## Decisions

### Decision: Model pull as an aggregate VMR command

The command should discover the VMR root, collect immediate child Git repositories, and invoke `git --no-optional-locks -C <repo> pull [<repository> [<refspec>...]]` in each child repository.

Alternative considered: treat pull like a plain Git passthrough from the effective working directory. That would update only one repository and would not solve the VMR repetition problem.

### Decision: Keep the initial argument surface to repository and refspecs

The parser should support one optional repository argument followed by zero or more refspec arguments. The first positional argument is always the Git pull repository argument, meaning `git vmr pull main` pulls from a repository or remote named `main`; it is not interpreted as a refspec for the default remote.

Alternative considered: collect arbitrary trailing arguments and forward them to Git. That would support more Git syntax immediately, but it weakens the documented contract, complicates help text and tests, and makes later option-specific behavior harder to specify.

### Decision: Delegate pull semantics and validation to Git

The command should not validate whether a child repository has a matching remote, configured upstream branch, clean working tree, compatible pull strategy, reachable repository URL, or valid refspec. Each child repository may have different remotes, branches, local changes, and pull configuration, so Git should decide success or failure per repository.

Alternative considered: preflight repository state before pulling. That could produce clearer failures for common cases, but it would duplicate Git semantics, risk diverging from local configuration, and still need to handle races between preflight and pull.

### Decision: Run pulls in parallel with buffered reporting

Pull attempts should run in parallel, with each worker capturing Git stdout and stderr. After all workers finish, results should be rendered in deterministic repository-name order.

Alternative considered: stream each child Git process directly to the terminal. That would preserve live progress, but parallel output would interleave and become hard to attribute to repositories. Buffered reporting is consistent with the existing aggregate command pattern.

### Decision: Report first useful Git line per repository

On success, if Git emits output, the command should print the first non-empty line from Git stdout, falling back to stderr, followed by the repository name. If Git emits no output, that repository contributes no success line. On failure, stderr should contain the first non-empty line from Git stderr, falling back to stdout, followed by the repository name.

Alternative considered: preserve full multi-line pull output for every repository. That keeps more detail, but it can become noisy in large VMRs and is harder to scan after parallel execution. A concise first-line report matches existing command behavior while leaving detailed investigation to running Git directly in the affected repository.

### Decision: Best-effort means no cross-repository transaction

If pull succeeds in one repository and fails or conflicts in another, the successful pull remains applied and the conflicted repository remains in Git's resulting state. The command should return a non-zero status when any repository fails, after attempting every discovered child Git repository.

Alternative considered: stop after the first failure or attempt rollback. Stopping early would leave users without a complete view of which repositories can update successfully. Rollback would require repository-specific recovery semantics and cannot reliably undo every Git pull outcome.

## Risks / Trade-offs

- [Risk] Captured output hides live pull progress and can make interactive credential prompts less usable -> Mitigation: keep v1 deterministic and document richer passthrough or streaming behavior as a possible future extension.
- [Risk] Parallel pulls can create conflicts in more than one repository at the same time -> Mitigation: delegate conflict behavior to Git and report every failed repository so users know where cleanup is required.
- [Risk] The same repository argument may mean different remotes or URLs per child repository -> Mitigation: delegate to Git and report per-repository failures.
- [Risk] First-line output can omit details from multi-line pull reports -> Mitigation: keep default output concise and leave detailed investigation to running Git directly in the affected repository.
- [Risk] Parallel network operations can stress shared remotes or credentials helpers -> Mitigation: match existing aggregate parallelism and keep repository attempts independent.
