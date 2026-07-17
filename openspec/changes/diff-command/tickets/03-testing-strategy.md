# Testing strategy for diff, colour, and paging

Type: grilling
Status: open
Blocked by: 02

## Question

How is `diff` tested? The scripted fake covers git invocations, but this command adds surfaces the fake doesn't reach:

- TTY detection driving `--color` and pager activation — how is it injected/faked?
- Pager spawning — is the pager behind a seam with a fake adapter (mirroring the Git runner pattern), or exercised only in integration tests?
- Asserting the combined output (prefixed paths, repo ordering, failure aggregation) against scripted diffs.

Depends on where the pager seam lands ([02-pager-seam](02-pager-seam.md)).

Note: ticket 02's answer (decision 5) already proposes positions on the first two bullets — TTY as an injected bool on the CLI context, no pager trait (the pager command string is the seam; emit tests pass `cat` or a recording script), resolution via the scripted fake answering `var GIT_PAGER`. This ticket confirms or overturns them and settles the third bullet, plus fixtures for the rename-rewrite transform (decision 6).
