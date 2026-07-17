## 1. Skipped repo outcome

- [x] 1.1 Add the `Skipped` variant (required reason) to `RepoOutcome` in `src/git`, alongside `Success`/`Failure`
- [x] 1.2 Carry skips through result aggregation: group by identical reason, repo-list suffix, stdout, no exit-code effect, quiet-success rule not applied
- [x] 1.3 Render skip lines dimmed via the existing rendering choke point
- [x] 1.4 Unit-test aggregation/rendering per the aggregate-command-output delta scenarios (grouping, exit code, all-skipped not silent)

## 2. Push classification

- [x] 2.1 Add head/config/remote-tracking evidence gathering to the push slice through the Git runner seam (local-only; scripted-fake coverage)
- [x] 2.2 Implement destination resolution for `push.default` (`simple`/`current`/`upstream`) and upstream config; unrecognized configuration classifies as skip-with-reason
- [x] 2.3 Implement the branch usefulness predicate: destination exists on target remote → delegate; novel commits (`rev-list --not --remotes=<remote>`) → push; otherwise skip as empty branch creation
- [ ] 2.4 Implement refspec parsing and the classification table: branch → predicate, delete → push, tag → push, wildcard → skip, URL repository argument → delegate whole push
- [ ] 2.5 Wire per-refspec filtering: pass only the useful subset in user order; no git invocation when the subset is empty; bare push as the one-refspec degenerate case
- [ ] 2.6 Unit-test each classification row and predicate branch against the scripted fake, including the deleted-on-remote re-creation case and stale-picture push-through

## 3. Spec conformance and integration

- [ ] 3.1 Update push integration tests: rewrite verbatim-delegation scenarios, add skip-reporting, mixed-refspec, tag/delete/wildcard/URL, and all-skipped-exit-0 scenarios from the delta spec
- [ ] 3.2 Verify skip reason wording aggregates well across repos (repo-independent phrasing) and reads clearly with the repo-list suffix
- [ ] 3.3 Run the full test suite and lints via mise tasks
