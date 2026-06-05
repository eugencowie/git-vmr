## Context

`git vmr mv` currently accepts exactly two path operands. The command discovers the VMR root, routes the source and destination to immediate child Git repositories, delegates same-repository moves to `git mv`, and synthesizes cross-repository moves with a filesystem rename plus staging in the source and destination repositories.

Git supports a second move form, `git mv <source>... <destination-directory>`, where the final operand must be an existing directory and every source is moved beneath it using its basename. `git-vmr` can expose the same shape while preserving its VMR routing rules and cross-repository staging model.

## Goals / Non-Goals

**Goals:**

- Support both `git vmr mv <source> <destination>` and `git vmr mv <source>... <destination-directory>`.
- Preserve current two-operand behavior and existing validation semantics.
- Allow multi-source moves whose sources and destination are owned by different child Git repositories.
- Validate all route ownership, destination directory shape, final destination conflicts, and duplicate final paths before moving anything.
- Delegate same-repository multi-source moves to Git where possible.

**Non-Goals:**

- Add `git mv` flags such as `--dry-run`, `-k`, `--force`, or `--verbose`.
- Provide transactional rollback for synthesized cross-repository moves after mutation starts.
- Move paths into or out of `.gitvmr/`, root-level VMR files, non-Git child directories, or nested repositories beyond the existing immediate-child ownership model.
- Preserve Git rename metadata across independent child repositories; cross-repository moves remain staged deletions and additions.

## Decisions

### Decision: Parse `mv` as two or more positional operands

The CLI should accept a single variadic positional list with at least two paths. The command layer should split the last operand as the destination and treat the preceding operands as sources.

For exactly one source, the command should keep the current rename behavior: the destination may be a final path or an existing directory. For more than one source, the destination is always a destination directory and must already exist.

Alternative considered: add separate `sources` and `destination` clap fields. Clap cannot naturally express "one or more sources followed by one destination" as clearly as a single operand vector, and the command layer still needs to split and validate the shape.

### Decision: Model moves as planned operations after full preflight

Before any filesystem or Git mutation, the command should build a plan for every source:

1. Route each source with `route_single_path`.
2. Route the destination with `route_single_path`.
3. For multi-source moves, require the routed destination path to be an existing directory.
4. Compute the final destination for each source as `<destination-directory>/<source-basename>`.
5. Reject duplicate final destinations and existing destination conflicts.
6. Validate source tracking before mutation, including synthesized cross-repository paths.

The existing two-operand path can reuse this planner by allowing the destination to be either a final path or an existing directory.

Alternative considered: move each source independently and let failures stop the loop. That would be simpler, but it would make obvious conflicts produce avoidable partial moves.

### Decision: Delegate all-same-repository plans to `git mv`

If every source and the destination are in the same child repository, invoke one Git command:

```sh
git -C <repo> mv -- <source>... <destination-directory>
```

for the multi-source form. This preserves Git's porcelain behavior for same-repository moves, including directory moves, platform path behavior, and index updates.

Alternative considered: implement all multi-source moves with filesystem operations and `git add`. That would duplicate Git behavior inside a child repository and create divergence risk for edge cases Git already handles.

### Decision: Synthesize cross-repository plans sequentially

When any source or final destination crosses child repository boundaries, execute the planned moves sequentially using the same model as the existing cross-repository single-source move: filesystem rename, stage source deletion, then stage destination addition.

The command should preflight every source before the first move, but it does not need to roll back already completed moves if a later filesystem or Git operation fails. This matches the current cross-repository non-transactional behavior.

Alternative considered: attempt rollback for previous moves. That would be brittle because staging may have partially succeeded, filesystem failures may be caused by external state, and rollback could create a second failure mode that obscures the original error.

### Decision: Reject duplicate source basenames targeting the same directory

For multi-source moves, two sources with the same basename would compute the same final destination. The command should reject that during preflight, even if the sources are from different child repositories.

Alternative considered: rely on the second move to fail because the destination exists. That would be too late for cross-repository moves and would violate the no-avoidable-partial-move goal.

## Risks / Trade-offs

- Cross-repository multi-source moves are still not atomic -> Preflight all predictable failures before mutation and report the failing path or repository if an operation fails after mutation starts.
- Same-repository and cross-repository behavior can diverge on Git edge cases -> Delegate all-same-repository moves to Git and keep synthesized behavior limited to cases Git cannot represent across independent repositories.
- Destination directory routing can be surprising for child repository roots -> Treat an existing child repository root as a valid destination directory, matching existing single-source behavior.
- Duplicate basenames are rejected even if a user might intend overwrites -> Overwrites remain out of scope until `--force` is supported.
