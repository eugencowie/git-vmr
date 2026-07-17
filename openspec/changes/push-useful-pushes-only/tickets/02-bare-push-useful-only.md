# 02 — Bare push attempts only useful pushes

**Status:** ready-for-agent

**What to build:** The core VMR workflow stops polluting remotes. A user creates a feature branch across the workspace, commits in two of five repos, and runs `git vmr push`: the two repos with novel commits push, the other three are skipped with a reported reason, and the command exits 0. A run in which every repo is skipped exits 0 and is not silent.

The push slice classifies each repo's bare push from local remote-tracking refs alone — never the network: evidence gathered through the Git runner seam (head, push-related config, remote-tracking refs), destination resolved the way `git push` would for the common `push.default` values and upstream config, and the usefulness predicate applied — destination exists on the target remote → delegate to Git; tip has novel commits → push; otherwise skip as an empty branch creation. Configuration the resolver does not understand skips with a reason (err toward skipping, per ADR 0007). The deleted-on-remote branch is not re-created; a stale local picture errs toward pushing. Detached and unborn heads may delegate or skip, whichever is simplest.

Decisions: ADR 0007 and the push-command delta spec in the push-useful-pushes-only change. Vocabulary: useful push, novel commits, empty branch (CONTEXT.md).

**Blocked by:** 01 — Skipped repo outcome through the aggregate pipeline.

- [ ] Bare `git vmr push` pushes only repos whose push is proven useful, skipping the rest with reasons
- [ ] Evidence is local remote-tracking refs and config only; no network before deciding
- [ ] A branch merged and deleted on the remote (upstream config lingering) is skipped, not re-created
- [ ] A stale local picture results in a push, which Git absorbs
- [ ] Unrecognized push configuration classifies as skip-with-reason
- [ ] Skipped repos incur no `git push` invocation; all-skipped runs exit 0 and report
- [ ] Scripted-fake unit tests cover each predicate branch; integration tests cover the workflow scenarios from the delta spec
