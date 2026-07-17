# 01 — Skipped repo outcome through the aggregate pipeline

**Status:** done

**What to build:** A fan-out command can report that it deliberately did not attempt a repo. The repo outcome gains a third variant, skipped, carrying a required reason, and it flows end-to-end through result aggregation and rendering: identical reasons group into one line with the repo-list suffix, the line prints dimmed on stdout, skips never affect the exit code, and the quiet-success rule does not apply — a run whose outcomes are all skipped exits 0 and is not silent. No command emits the variant yet; this is the prefactor that lets the push filter tickets land as pure push-slice work.

Decisions and vocabulary: CONTEXT.md (repo outcome, result aggregation, repo-list suffix, rendering) and ADR 0007. Requirements: the aggregate-command-output delta spec in the push-useful-pushes-only change.

**Blocked by:** None — can start immediately.

- [x] Repo outcome has a skipped variant with a required reason, alongside success and failure
- [x] Result aggregation groups skips by identical reason with the repo-list suffix, on stdout
- [x] Skip lines render dimmed through the existing rendering choke point
- [x] Skips never affect the exit code; an all-skipped run exits 0 and prints its skip reasons
- [x] Unit tests cover the aggregate-command-output delta scenarios (grouping, exit code, all-skipped not silent)
