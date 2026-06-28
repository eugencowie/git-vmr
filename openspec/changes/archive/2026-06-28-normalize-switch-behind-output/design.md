## Context

Fan-out Git commands return structured repository outcomes and the shared renderer groups exact success and failure messages. `git switch` can print successful advisory lines to stdout before its `Switched to branch ...` stderr line, and `git-vmr` intentionally selects stdout first for switch successes.

For repositories with the same advisory shape but different commit counts, Git emits messages that only differ in the `by N commit(s)` phrase. Exact grouping treats those as different messages, creating noisy output in large VMRs.

## Goals / Non-Goals

**Goals:**

- Group equivalent `git switch` advisories across repositories with different commit counts.
- Preserve the upstream branch name and surrounding Git wording.
- Keep normalization local to successful switch messages.
- Use a focused regex-based normalization for the variable commit-count phrase.

**Non-Goals:**

- Do not add fuzzy matching to the shared aggregate renderer.
- Do not normalize diverged, failure, or create-and-switch output.
- Do not change switch execution, branch resolution, or exit status behavior.

## Decisions

### Decision: Normalize in the switch adapter before creating the success outcome

Add a small private helper in `src/git/switch.rs` that receives the selected success message and returns the normalized form. The helper should run only on `git::switch`, not on the shared renderer and not on failed outcomes.

This keeps the global aggregate renderer exact by default while letting known-safe Git advisories collapse into one group.

Alternative considered: normalize inside `grouped_messages`. That would affect every fan-out command and make command-specific Git wording leak into generic rendering.

### Decision: Strip only the variable commit-count phrase

Recognize selected successful switch messages containing:

```text
 by N commit(s)
```

Render them without that variable fragment:

```text
Your branch is behind '<upstream>', and can be fast-forwarded.
Your branch is ahead of '<upstream>'.
```

The implementation uses a compiled `regex::Regex` to strip ` by \d+ commits?` from selected successful switch messages. This keeps normalization local to `git::switch` while preserving the upstream branch name and surrounding Git wording.

Alternative considered: replace the number with a placeholder such as `by N commits`. Removing the phrase reads cleaner and matches the requested output merge.

## Risks / Trade-offs

- [Risk] Future Git versions may alter advisory wording. -> Mitigation: only the specific commit-count fragment is removed; other text remains unchanged.
- [Risk] Removing the count hides per-repository behind distance in the summary. -> Mitigation: users can run `git vmr status`, `git fetch`, or per-repo Git commands when the exact count matters; the switch summary's job is concise aggregate reporting.
