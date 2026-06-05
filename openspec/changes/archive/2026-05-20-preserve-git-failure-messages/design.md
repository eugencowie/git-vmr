## Context

Fan-out Git commands capture per-repository stdout, stderr, and exit status, then render grouped messages with repository suffixes. The current failure path has two manual `fatal:` handling points: selected Git lines may be stripped with fatal-specific helpers, and aggregate errors may be printed through a formatter that conditionally adds `fatal:` back.

This behavior was introduced to avoid duplicated `fatal:` prefixes after aggregation. The new direction is simpler: the selected Git output line is the message. If Git includes `fatal:`, the user sees it. If Git includes `error:`, the user sees it. If a fallback message is used because Git emitted no line, the fallback remains a tool-authored message.

## Goals / Non-Goals

**Goals:**

- Preserve Git-emitted failure text exactly after the existing first-line and stderr/stdout fallback selection.
- Remove code that strips or adds the `fatal:` prefix for fan-out Git failures.
- Keep aggregate grouping, repository suffix formatting, deterministic ordering, quiet successes, and non-zero failure exit semantics unchanged.
- Update tests and specs so they assert Git's emitted prefixes instead of tool-normalized prefixes.

**Non-Goals:**

- Do not change which stream is preferred for each command when selecting a message.
- Do not change top-level non-aggregate errors that are not captured Git fan-out output.
- Do not preserve multi-line Git output; existing first-non-empty-line behavior remains in scope.
- Do not alter CLI arguments or add configuration for output formatting.

## Decisions

### Decision: Preserve selected Git messages at capture time

Commands should use the existing non-stripping helpers (`first_non_empty_line` and `first_non_empty_line_with_fallback`) or equivalent direct stderr/stdout capture. This keeps the Git adapter boundary responsible only for selecting a message, not rewriting its prefix.

Alternative considered: keep stripping in command adapters and stop adding `fatal:` in `main`. That would remove duplicate prefixes but would still discard Git-emitted `fatal:` text, which conflicts with preserving what Git outputs.

### Decision: Print aggregate failures without fatal-prefix formatting

`AggregateError` entries should already be render-ready grouped messages. `main` should print each aggregate error as-is instead of routing through fatal-prefix logic.

Alternative considered: keep a conditional top-level formatter. That would keep behavior for fallback messages close to today, but it preserves the special case the change is intended to remove.

### Decision: Keep fallback messages plain

Fallback messages such as `git branch failed` are not Git output and should not receive a synthetic `fatal:` prefix through aggregate rendering. They remain rare fallback text used when Git emits no selected line.

Alternative considered: add `fatal:` only to fallback messages. That would require classifying message origin throughout aggregation and would reintroduce manual prefix policy.

## Risks / Trade-offs

- Existing scripts or tests that expect every fan-out failure line to begin with `fatal:` may need updates. Mitigation: update project tests and specs to describe exact Git-message preservation.
- Different Git versions may vary message prefixes. Mitigation: the command already delegates wording to Git; tests should use stable scenarios and avoid asserting prefixes Git does not emit.
- Fallback messages will be less Git-like because they no longer receive `fatal:`. Mitigation: fallback messages are only used when Git emits no meaningful output and should remain descriptive.
