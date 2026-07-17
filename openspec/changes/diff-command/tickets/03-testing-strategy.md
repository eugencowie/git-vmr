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
