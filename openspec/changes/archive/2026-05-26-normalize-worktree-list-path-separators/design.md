## Context

`git vmr status` already renders paths with Git-style `/` separators by converting the display string before writing output. `git vmr worktree list` currently renders aggregate roots with `Path::display()`, so a Windows path sourced from filesystem APIs can appear with `\` while a path sourced from Git porcelain output can appear with `/`.

The worktree list command already computes aggregate roots as `PathBuf`s, groups them deterministically, and only converts them to text at render time. That makes the rendering boundary the smallest place to normalize separators without changing grouping, filtering, or Git command behavior.

## Goals / Non-Goals

**Goals:**

- Render every aggregate worktree path in `git vmr worktree list` with `/` separators.
- Match the Git-style path formatting used by `git vmr status`.
- Keep worktree grouping, deterministic ordering, branch rendering, detached HEAD rendering, and repository suffix rendering unchanged.
- Cover Windows-style separator input in tests in a host-independent way.

**Non-Goals:**

- Do not change how paths are parsed from `git worktree list --porcelain -z`.
- Do not change aggregate root discovery or filtering.
- Do not change CLI arguments, output shape beyond path separators, or exit statuses.
- Do not canonicalize, absolutize, relativize, or otherwise rewrite paths beyond separator normalization.

## Decisions

### Decision: Normalize only at the output rendering boundary

The implementation should keep `PathBuf` values for path comparisons, sorting, and grouping, then convert separators when rendering the path text. This avoids changing behavior that depends on platform path semantics while fixing the user-visible inconsistency.

Alternative considered: normalize paths immediately after parsing Git porcelain output. That would spread display-specific behavior into data collection and could affect path comparisons on platforms where backslashes are ordinary path characters.

### Decision: Prefer a small display helper consistent with status

The worktree renderer should use a helper equivalent to status path rendering: convert `Path::display()` to a string and replace `\` with `/`. The helper may remain local to the worktree command or be shared if doing so stays simple.

Alternative considered: use a path component iterator to rebuild paths with `/`. That is unnecessary for this case and can handle Windows drive prefixes awkwardly on non-Windows hosts.

### Decision: Test separator normalization directly

Add a focused test using a Windows-style path such as `C:\Projects\vmr` and assert that rendered output contains `C:/Projects/vmr`. A direct render/helper test keeps the behavior enforceable on non-Windows CI.

Alternative considered: rely only on an end-to-end integration test. That would be hard to make portable because real worktree paths are host-dependent.

## Risks / Trade-offs

- [Risk] Replacing backslashes in display output can alter paths where `\` is a literal filename character on Unix-like systems. -> Mitigation: this command is matching Git-style display conventions already used by `status`; the change is intentionally limited to rendered CLI output, not path operations.
- [Risk] Duplicating the status helper could leave two implementations to maintain. -> Mitigation: the helper is trivial; share it only if the resulting module boundary stays cleaner than local duplication.
- [Risk] Tests could couple to private rendering internals. -> Mitigation: keep the test focused on the observable rendered path string and avoid asserting unrelated list formatting.
