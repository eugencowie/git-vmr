## Context

`git vmr add` already discovers the VMR root, resolves path arguments relative to the effective working directory, routes each path to an immediate child Git repository, and invokes `git add -- <paths...>` per repository. A path resolving to the VMR root already expands to `.` in every immediate child Git repository, while explicit invalid ownership fails before any repository is staged.

The requested flags map to existing Git add behavior but introduce one VMR-specific ambiguity: Git allows `git add -A` without pathspecs, while the current VMR command requires at least one path. In a virtual monorepo, no-path all-mode needs a clear aggregate meaning.

## Goals / Non-Goals

**Goals:**
- Support `git vmr add -A`, `git vmr add --all`, `git vmr add -f`, `git vmr add --force`, and `git vmr add --chmod=(+|-)x`.
- Preserve existing path routing, root expansion, and preflight validation for explicit pathspecs.
- Define no-path `-A` and `--all` as aggregate staging across every immediate child Git repository.
- Keep option parsing explicit and testable.

**Non-Goals:**
- Supporting additional `git add` options such as `--update`, `--intent-to-add`, `--renormalize`, `--pathspec-from-file`, interactive modes, or generic passthrough.
- Supporting full Git pathspec magic beyond the existing literal-first routed path behavior.
- Adding repository inclusion/exclusion configuration.
- Making multi-repository staging transactional after Git execution begins.

## Decisions

### Decision: Treat no-path all-mode as aggregate VMR staging

When `-A` or `--all` is present without pathspecs, the command will stage `.` in every immediate child Git repository discovered from the VMR root, regardless of whether the effective working directory is the VMR root or inside a child repository. Non-Git child directories are skipped.

Rationale: `git vmr` commands present the VMR as an aggregate workspace. Existing `git vmr add .` at the VMR root already means all child repositories, so `git vmr add -A` should be the option-driven spelling of that aggregate all-staging behavior.

Alternative considered: scope no-path `-A` to the current child repository when invoked from inside a child. That would be closer to plain Git's current-working-repository behavior, but it would make `git vmr add -A` context-sensitive in a surprising way and less useful as a VMR-level command.

### Decision: Keep pathspecs required unless all-mode is present

The parser should accept zero pathspecs only when `-A` or `--all` is present. `git vmr add` with no flags and no pathspecs should remain invalid.

Rationale: The existing command requires explicit staging targets. Loosening that to a no-op or implicit all-mode without `-A` would be a behavioral change unrelated to the requested Git flags.

Alternative considered: accept `git vmr add` as an alias for `git vmr add -A`. Git does not do that, and accidental bare invocation would become mutating.

### Decision: Model supported flags explicitly rather than adding passthrough

The CLI should add typed fields for `all`, `force`, and `chmod`, then the Git wrapper should construct `git add` arguments in a fixed order before `--` and routed paths.

Rationale: Explicit support keeps documentation, help, validation, and tests aligned with the narrow compatibility surface. It also prevents unsupported Git add flags from bypassing VMR routing assumptions.

Alternative considered: collect arbitrary `git add` arguments before pathspecs and pass them through. That would be more flexible, but it would blur which options are supported and introduce routing ambiguity for options that affect pathspec parsing or input sources.

### Decision: Restrict chmod values to executable-bit forms

The parser should accept only `--chmod=+x` and `--chmod=-x`, matching Git's supported executable-bit values. The equals form should be documented and tested.

Rationale: `--chmod=-x` begins with a hyphen in the value, so requiring the equals form avoids ambiguous CLI parsing and mirrors the documented Git syntax.

Alternative considered: accept `--chmod +x` as well. That adds parser edge cases without improving parity for the documented form.

### Decision: Preserve VMR validation before mutation

`--force` should only affect Git's ignored-file behavior after a path has been routed to a child Git repository. It must not allow paths outside the VMR, paths inside `.gitvmr/`, VMR-root files, or explicit non-Git child paths to proceed.

Rationale: The force flag belongs to Git add semantics, not VMR ownership semantics. Keeping validation first avoids accidental staging outside the virtual monorepo model.

Alternative considered: let `--force` weaken VMR validation. That would make the flag broader than Git's meaning and undermine the command's safety contract.

## Risks / Trade-offs

- **No-path all-mode can stage many repositories** -> Document it clearly and add integration coverage from both VMR root and child working directories.
- **Multi-repo staging remains non-transactional** -> Keep preflight routing before mutation and rely on existing per-repository failure reporting for Git errors.
- **`--chmod` on invalid or missing paths delegates nuanced behavior to Git** -> Route ownership first, then preserve Git's own error messages through existing failure reporting.
- **Parallel repository execution can produce multiple independent Git failures** -> Keep collecting results through existing aggregation so failures remain deterministic enough for the current output model.
