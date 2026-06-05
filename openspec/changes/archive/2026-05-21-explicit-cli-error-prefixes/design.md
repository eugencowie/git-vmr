## Context

The entry point currently treats errors in two categories: `AggregateError` values are downcast and printed verbatim line by line, while every other error is printed as `fatal: {e:#}`. This preserves fan-out Git output after the failure-message preservation change, but it still makes direct `git-vmr` errors inherit a global `fatal:` prefix regardless of the failure category.

Git itself uses mixed user-facing prefixes. Some failures are `fatal:`, some are `error:`, some are command-prefixed, and hook or remote output can be unprefixed. To match that style, `git-vmr` needs rendered direct errors to carry their chosen prefix before they reach `main`.

## Goals / Non-Goals

**Goals:**

- Make top-level error rendering verbatim for all errors.
- Move `git-vmr` authored prefix selection to the error creation or command boundary.
- Preserve aggregate fan-out Git messages exactly as selected and grouped.
- Preserve non-zero exit status for direct and aggregate failures.
- Keep the change local to rendering and error construction; do not alter CLI arguments or command behavior.

**Non-Goals:**

- Do not rewrite Git-emitted stderr/stdout lines.
- Do not introduce a configuration option for error prefix style.
- Do not change Clap-generated parse errors.
- Do not change which failure line each fan-out command selects.

## Decisions

### Decision: Render top-level errors verbatim

`main` should print `{e:#}` without adding `fatal:`. This makes the entry point a transport boundary rather than a policy boundary.

Alternative considered: keep the current `fatal:` fallback and only special-case more aggregate-like errors. That preserves current behavior for direct errors but keeps prefix policy centralized in a way that cannot express `error:` or unprefixed direct failures.

### Decision: Use inline rendered prefixes for tool-authored errors

Direct `git-vmr` authored errors should include their rendered prefix directly in the user-facing error string, for example `fatal: ...` or `error: ...`, at the creation or command boundary. This keeps the rendering contract explicit without adding a dedicated error abstraction for a small set of prefixes.

Alternative considered: introduce a lightweight rendered-error type. That would centralize prefix construction, but it adds an abstraction without enough benefit for the current codebase. The tests guard the important behavior: no blanket top-level prefixing and no accidental double-prefixing.

### Decision: Classify direct errors during implementation audit

Direct errors should be audited and classified by behavior:

- Environment, discovery, invocation, and unrecoverable setup failures use `fatal:`.
- Tool-authored operand or validation failures may use `error:` when they match Git's non-fatal validation style.
- Git-emitted or hook-emitted output is preserved verbatim and not classified by `git-vmr`.

The implementation should update tests alongside each classification so the contract is captured by behavior rather than implementation style.

### Decision: Replace aggregate error downcast with joined render-ready errors

Once `main` prints errors verbatim, `print_results` can join rendered aggregate failure lines with newlines and return them as a single error value. This removes the need for a top-level aggregate downcast while preserving aggregate stderr exactly.

Alternative considered: keep `AggregateError` as a special type even after verbatim top-level rendering. That is safe but no longer necessary if the joined display text is already render-ready and no prefix injection remains.

## Risks / Trade-offs

- Missed direct error classification could produce unprefixed stderr. Mitigation: audit all direct `bail!`, `.context(...)`, and propagated errors that can reach `main`, and add representative tests.
- Over-classification could add prefixes to errors that later flow through aggregate rendering. Mitigation: classify only direct top-level paths and leave `RepoOutcome::Failure` messages verbatim.
- `anyhow` context strings can produce multi-part messages. Mitigation: put the chosen prefix on the outermost user-facing context when wrapping external errors.
- Existing tests that assert `fatal:` for direct failures will need updates if a failure is intentionally reclassified as `error:`.

## Migration Plan

1. Add inline rendered prefixes to direct CLI errors and update `main` to print errors verbatim.
2. Convert aggregate rendering to return joined render-ready failure text.
3. Audit direct top-level error paths and apply explicit inline `fatal:` or `error:` prefixes.
4. Update or add tests for representative direct and aggregate rendering cases.
5. Run the Rust test suite and OpenSpec validation.

## Open Questions

- Which specific direct validation failures should move from `fatal:` to `error:` during the first implementation pass?
- Should prefix strings remain inline long-term, or should a rendered-error abstraction be introduced later if more rendering categories appear?
