## Context

Most mutating `git-vmr` commands fan out to immediate child Git repositories, capture one concise Git output line per repository, and render those results after all repository attempts finish. The current shared result type is `Result<Option<String>>`, so Git adapters usually append the repository name while constructing the string. The CLI renderer then prints every success string to stdout and returns an aggregate error that `main` renders one error per line.

That design is simple, but it prevents useful aggregation. By the time results reach the renderer, the output line and repository identity are already combined, so identical Git messages from many repositories no longer compare equal. This also contributes to duplicate `fatal:` prefixes because Git may emit a fatal line and `main` adds its own fatal prefix.

## Goals / Non-Goals

**Goals:**

- Preserve Git's concise output as the primary user-facing message.
- Group exact duplicate Git messages across repositories for both successes and failures.
- Keep repository identity available in grouped suffixes so failures remain actionable.
- Keep quiet successes quiet when Git emits no output.
- Keep best-effort fan-out semantics: continue attempting all eligible repositories and fail the overall command if any child operation fails.
- Centralize aggregate output rendering so command adapters do not each implement their own string policy.

**Non-Goals:**

- Do not introduce fuzzy matching or command-specific similarity heuristics.
- Do not stream child Git output while parallel operations are still running.
- Do not add rollback behavior for partial failures.
- Do not add a verbose flag in this change, though the structure should leave room for one later.
- Do not change non-fan-out commands such as `clone`, `init`, or cross-repository `mv` operations that do not use aggregate result rendering.

## Decisions

### Decision: Preserve structured repository outcomes until rendering

Replace the rendered-string result path for fan-out Git operations with a structured outcome containing repository name, status, and optional raw Git message. The renderer should receive data equivalent to:

```rust
struct RepoMessage {
    repo: String,
    message: String,
}

enum RepoOutcome {
    Success(Option<RepoMessage>),
    Failure(RepoMessage),
}
```

The exact Rust shape can be adjusted during implementation, but the important boundary is that Git adapters should not append ` (repo)` to the message string. Repository context belongs to the aggregate renderer.

Alternative considered: keep returning rendered strings and strip trailing ` (repo)` before grouping. That is brittle because Git messages can legitimately contain parenthesized text, and it continues to spread presentation decisions into adapters.

### Decision: Group by exact normalized message and outcome status

The renderer should group successes separately from failures, using the exact message string after minimal fatal-prefix normalization. Success and failure groups should not merge even if their text is the same.

Exact grouping is predictable and avoids accidentally hiding meaningful differences in refs, paths, hashes, remotes, or Git advice. Similarity grouping can be added later as an explicit command-specific normalization layer if real output shows safe opportunities.

Alternative considered: use fuzzy matching to group similar Git lines. That would reduce more noise, but it risks collapsing messages that differ in important details.

### Decision: Render one line per unique message with grouped repository context

For a single repository, preserve the familiar shape:

```text
Already up to date. (backend)
```

For multiple repositories with the same message, render one grouped suffix:

```text
Already up to date. (backend, frontend)
```

For large success groups, the renderer may use a count-only suffix to avoid replacing repeated message spam with a very long repository list:

```text
Already up to date. (48 repos)
```

Failures should remain more actionable than successes. A failure group should include repository names when practical:

```text
'origin' does not appear to be a git repository (alpha, zeta)
```

If the failure group is too large for a concise line, the renderer should still make the count explicit and include enough repository context to identify the affected set, for example:

```text
'origin' does not appear to be a git repository (48 repos: alpha, backend, frontend, ...)
```

The exact threshold can be implemented as a small private constant and covered with renderer tests.

Alternative considered: always list all repository names. That is best for traceability, but a 50-repository suffix is still noisy for common success messages.

### Decision: Render fatal prefix once

The aggregate error path should not blindly add `fatal:` in front of a message that already begins with `fatal:`. Either the Git adapter should strip Git's `fatal: ` prefix while preserving the rest of the message, or the final renderer should add the prefix conditionally. The renderer boundary is preferable because it keeps raw Git message capture simple and makes the final output rule testable in one place.

Alternative considered: keep the current `main`-level prefix behavior. That preserves existing code shape, but produces duplicated prefixes for many Git errors.

## Risks / Trade-offs

- [Risk] Exact grouping misses messages that are similar but not identical. -> Mitigation: start conservative; add explicit normalization only for well-understood command outputs.
- [Risk] Count-only success groups hide which repositories succeeded. -> Mitigation: failures keep repository names, command state remains inspectable via follow-up commands, and a future verbose mode can expose full per-repo output.
- [Risk] Changing result types touches many Git adapters and tests. -> Mitigation: implement a small shared conversion helper for `GitOutput` to `RepoOutcome` and update command tests around grouped renderer behavior.
- [Risk] Existing command specs describe per-repository output. -> Mitigation: introduce the shared aggregate-output capability first, then reconcile command-specific specs as part of archiving or a follow-up spec cleanup.
