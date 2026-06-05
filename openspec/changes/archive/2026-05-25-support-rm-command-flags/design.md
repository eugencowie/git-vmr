## Context

`git vmr rm` already discovers the VMR root from the effective working directory, validates and routes user-facing paths to immediate child Git repositories, and delegates removal semantics to `git rm` once per affected repository. The current implementation supports `-r` but explicitly left `--force`, `--dry-run`, and `--cached` out of the initial command scope.

These requested flags map directly to Git's own `rm` options, but they need to remain inside the existing VMR safety model. Flags can change Git removal behavior after a path is routed, but they must not weaken VMR ownership validation or allow mutation outside immediate child repositories.

## Goals / Non-Goals

**Goals:**
- Support `git vmr rm -f`, `git vmr rm --force`, `git vmr rm -n`, `git vmr rm --dry-run`, and `git vmr rm --cached`.
- Preserve existing path routing, root expansion, and preflight ownership validation.
- Keep `git vmr rm -n` useful by surfacing Git's dry-run removal output.
- Preserve Git's own force, dry-run, cached, and recursive semantics once paths have been routed to a child repository.

**Non-Goals:**
- Supporting additional `git rm` flags such as `--ignore-unmatch`, `--quiet`, `--sparse`, pathspec-from-file, or generic passthrough.
- Changing the literal-first VMR path routing model into full Git pathspec routing.
- Making multi-repository removal transactional after Git execution begins.
- Loosening VMR-root removal protections for dry-run or force mode.

## Decisions

### Decision: Model the new flags as explicit parser fields

Add typed `force`, `dry_run`, and `cached` booleans to the `Rm` command and pass them through command dispatch to the Git rm wrapper. The Git wrapper should construct a fixed argument list such as `git rm [--recursive] [--force] [--dry-run] [--cached] -- <paths...>`.

Rationale: explicit fields keep CLI help, tests, docs, and supported behavior aligned. This follows the recent `git vmr add` flag work and avoids exposing unsupported Git options that could conflict with VMR routing assumptions.

Alternative considered: collect arbitrary rm options and pass them through. That would be more flexible but would blur the supported compatibility surface and make path ownership rules harder to reason about.

### Decision: Preserve VMR validation before applying flag semantics

The command should continue to call VMR routing and validation before any `git rm` invocation. `--force` should only affect Git's tracked-file safety checks. `--cached` should only affect whether Git removes working tree files after routing. `--dry-run` should only affect whether Git mutates after routing.

Rationale: these flags belong to Git's repository-local behavior, not to VMR ownership. Keeping validation first preserves the existing safety contract that no invalid path can partially mutate another repository in the same command.

Alternative considered: allow `--force` to bypass some VMR validation. That would make force broader than Git's meaning and could allow destructive operations outside the virtual monorepo model.

### Decision: Keep VMR-root expansion gated by recursive intent

A path resolving to the VMR root should continue to require `-r`, even with `--dry-run`. Users who want to preview aggregate root removal can run `git vmr rm -n -r .`.

Rationale: root expansion targets every immediate child Git repository. Requiring `-r` keeps that broad scope explicit for both mutating and preview modes, and it avoids adding a separate dry-run-only code path that reports a command shape which would later fail when run without dry-run.

Alternative considered: allow `git vmr rm -n .` without `-r` because it does not mutate. That makes dry-run less faithful to the mutating command and weakens an important guardrail around aggregate root targeting.

### Decision: Surface successful dry-run output

Normal successful `git vmr rm` invocations should remain quiet. When `dry_run` is true, the Git wrapper should convert non-empty successful stdout from `git rm --dry-run` into success messages that flow through the existing repository-suffixed aggregate output path.

Rationale: `git rm -n` is useful because it prints what would be removed. Suppressing stdout would make the flag technically accepted but operationally weak. Reusing existing aggregate result rendering keeps multi-repository output consistent with other commands that surface child Git output.

Alternative considered: always print Git stdout for rm. That would change the existing quiet success behavior for non-dry-run removals.

## Risks / Trade-offs

- **[Risk] Dry-run output formatting can vary slightly by Git version** -> Test for stable markers such as removed path names and repository context rather than exact full output when needed.
- **[Risk] Multiple repositories can produce interleaved conceptual previews** -> Continue collecting per-repository results and rendering through the existing deterministic aggregate output model.
- **[Risk] `--cached` can expose Git's nuanced staged/working-tree comparison rules** -> Delegate to Git after routing and preserve Git failure messages, matching the existing command philosophy.
- **[Risk] Users may expect `--force` to override VMR validation** -> Document and test that force is repository-local and does not bypass VMR ownership checks.
