# 03 — Explicit refspecs are classified and filtered per refspec

**Status:** ready-for-agent

**What to build:** Explicit arguments get the same protection as bare push, per refspec. `git vmr push origin main feature` pushes, in each repo, only the useful subset of the typed refspecs in user order — a repo where `main` has novel commits but `feature` would be an empty branch creation runs `git push origin main` and reports the `feature` skip; a repo with no useful refspecs gets no git invocation. Bare push becomes the one-refspec degenerate case of the same mechanism.

The classification table lands whole: branch refspecs get the usefulness predicate; delete refspecs always push; tag refspecs always push with no check (locally unverifiable, explicit intent); wildcard refspecs skip with a reason as unclassifiable; a URL repository argument delegates the entire push unfiltered. There is no override flag — `git vmr foreach 'git push'` is the fallback.

Also closes out the spec change: the old verbatim-delegation integration scenarios are rewritten to the new requirements, and skip-reason wording is polished so identical reasons aggregate cleanly across repos.

Decisions: ADR 0007 and the push-command delta spec in the push-useful-pushes-only change.

**Blocked by:** 02 — Bare push attempts only useful pushes.

- [ ] Each repo pushes only its useful refspec subset, preserving user order; empty subset means no invocation
- [ ] Delete refspecs always push; tag refspecs always push without a usefulness check
- [ ] Wildcard refspecs skip with a reason naming the refspec as unclassifiable
- [ ] A URL repository argument passes the whole push through unfiltered
- [ ] Mixed-refspec skips are reported per refspec alongside the repo's push result
- [ ] Integration tests cover the classification and mixed-refspec scenarios from the delta spec; verbatim-delegation scenarios are rewritten
- [ ] Full test suite and lints pass via mise tasks
