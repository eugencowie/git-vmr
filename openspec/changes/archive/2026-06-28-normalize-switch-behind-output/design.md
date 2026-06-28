## Context

Fan-out Git commands return structured repository outcomes and the shared renderer groups exact success and failure messages. `git switch` can print a successful behind-and-fast-forward advisory to stdout before its `Switched to branch ...` stderr line, and `git-vmr` intentionally selects stdout first for switch successes.

For repositories behind the same upstream by different amounts, Git emits messages that only differ in the `by N commit(s)` phrase. Exact grouping treats those as different messages, creating noisy output in large VMRs.

## Goals / Non-Goals

**Goals:**

- Group equivalent `git switch` behind-and-fast-forward advisories across repositories with different behind counts.
- Preserve the upstream branch name and Git's fast-forward wording.
- Keep normalization local to successful switch messages.
- Use a focused regex-based normalization for the variable behind-count phrase.

**Non-Goals:**

- Do not add fuzzy matching to the shared aggregate renderer.
- Do not normalize ahead, diverged, failure, or create-and-switch output.
- Do not change switch execution, branch resolution, or exit status behavior.

## Decisions

### Decision: Normalize in the switch adapter before creating the success outcome

Add a small private helper in `src/git/switch.rs` that receives the selected success message and returns the normalized form. The helper should run only on `git::switch`, not on the shared renderer and not on failed outcomes.

This keeps the global aggregate renderer exact by default while letting the one known-safe Git advisory collapse into one group.

Alternative considered: normalize inside `grouped_messages`. That would affect every fan-out command and make command-specific Git wording leak into generic rendering.

### Decision: Strip only the variable behind-count phrase

Recognize messages shaped like:

```text
Your branch is behind '<upstream>' by N commit(s), and can be fast-forwarded.
```

Render them as:

```text
Your branch is behind '<upstream>', and can be fast-forwarded.
```

The implementation uses a compiled `regex::Regex` to strip ` by \d+ commits?` from selected successful switch messages. This keeps normalization local to `git::switch` while preserving the upstream branch name and Git's fast-forward wording.

Alternative considered: replace the number with a placeholder such as `by N commits`. Removing the phrase reads cleaner and matches the requested output merge.

## Risks / Trade-offs

- [Risk] Future Git versions may alter the advisory wording. -> Mitigation: leave non-matching messages unchanged.
- [Risk] Removing the count hides per-repository behind distance in the summary. -> Mitigation: users can run `git vmr status`, `git fetch`, or per-repo Git commands when the exact count matters; the switch summary's job is concise aggregate reporting.
