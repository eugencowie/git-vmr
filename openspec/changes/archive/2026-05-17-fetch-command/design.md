## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing aggregate commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository-local semantics, and report repository names when rendering aggregate output.

`git vmr fetch [<repository> [<refspec>...]]` should follow that aggregate model. Unlike `clone`, fetch operates on existing child repositories and should therefore require VMR root discovery. Unlike quiet commands such as `rebase`, fetch can produce useful successful output when remote-tracking refs change, so the command needs deterministic success reporting as well as deterministic failure reporting.

## Goals / Non-Goals

**Goals:**

- Add `git vmr fetch [<repository> [<refspec>...]]` for fetching in every immediate child Git repository.
- Preserve Git's interpretation of the optional repository and refspec arguments in each child repository.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository fetch attempts in parallel.
- Report successful fetch output and failed fetch output with repository suffixes in deterministic repository-name order.
- Reuse existing global `-C <path>` and VMR root discovery behavior.

**Non-Goals:**

- Do not add fetch option flags such as `--all`, `--prune`, `--tags`, `--depth`, or generic trailing passthroughs in the initial command.
- Do not pre-resolve remote names, repository URLs, or refspecs before invoking Git.
- Do not synchronize remote configuration or rewrite repository arguments across child repositories.
- Do not recursively discover nested repositories or add configured repository inclusion/exclusion.
- Do not provide live per-repository progress streaming in the initial implementation.

## Decisions

### Decision: Model fetch as an aggregate VMR command

The command should discover the VMR root, collect immediate child Git repositories, and invoke `git --no-optional-locks -C <repo> fetch [<repository> [<refspec>...]]` in each child repository.

Alternative considered: treat fetch like `clone` and pass through to Git from the effective working directory. That would be closer to plain `git fetch`, but it would not solve the VMR repetition problem because it would fetch only one repository.

### Decision: Keep the initial argument surface to repository and refspecs

The parser should support one optional repository argument followed by zero or more refspec arguments. The first positional argument is always the Git fetch repository argument, meaning `git vmr fetch main` fetches from a repository or remote named `main`; it is not interpreted as a refspec for the default remote.

Alternative considered: collect arbitrary trailing arguments and forward them to Git. That would support more Git syntax immediately, but it weakens the documented contract, complicates help text and tests, and makes later option-specific behavior harder to specify.

### Decision: Delegate repository and refspec validation to Git

The command should not validate whether a child repository has a remote named by the repository argument, whether a repository URL is reachable, or whether refspecs match any remote refs. Each child repository may have different remotes and refs, so Git should decide success or failure per repository.

Alternative considered: preflight remote existence in every repository. That could produce clearer failures for simple remote-name cases, but it would incorrectly constrain valid Git fetch inputs such as repository URLs and complex refspecs.

### Decision: Run fetches in parallel with buffered reporting

Fetch attempts should run in parallel, with each worker capturing Git stdout and stderr. After all workers finish, results should be rendered in deterministic repository-name order.

Alternative considered: stream each child Git process directly to the terminal. That would preserve live progress, but parallel output would interleave and become hard to attribute to repositories. Buffered reporting is consistent with the existing aggregate command pattern.

### Decision: Report first useful Git line per repository

On success, if Git emits output, the command should print the first non-empty line from Git stderr, falling back to stdout, followed by the repository name. If Git emits no output, that repository contributes no success line. On failure, stderr should contain the first non-empty line from Git stderr, falling back to stdout, followed by the repository name.

Alternative considered: preserve full multi-line fetch output for every repository. That keeps more detail but can become noisy in large VMRs. A concise first-line report matches existing command behavior and keeps the aggregate output scannable.

### Decision: Best-effort means no cross-repository transaction

If fetch succeeds in one repository and fails in another, the successful fetch remains applied. The command should return a non-zero status when any repository fails, after attempting every discovered child Git repository.

Alternative considered: stop after the first failure. That would reduce remote traffic after an error, but it would leave users without a complete view of which repositories can fetch successfully.

## Risks / Trade-offs

- [Risk] Captured output hides live fetch progress and can make interactive credential prompts less usable -> Mitigation: keep v1 deterministic and document richer passthrough or streaming behavior as a possible future extension.
- [Risk] The same repository argument may mean different remotes or URLs per child repository -> Mitigation: delegate to Git and report per-repository failures.
- [Risk] First-line output can omit details from multi-line fetch reports -> Mitigation: keep default output concise and leave detailed investigation to running Git directly in the affected repository.
- [Risk] Parallel network operations can stress shared remotes or credentials helpers -> Mitigation: match existing aggregate parallelism and keep repository attempts independent.
