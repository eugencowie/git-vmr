---
status: accepted
---

# Git invocation goes through a runner seam, not an operation-level trait

Every git invocation passes through a single Git runner seam: a two-method interface (run captured, run interactive) satisfied by a subprocess adapter in production and a scripted fake in tests. The git module above the seam — arg construction, porcelain parsing, message-extraction rules — is real code in both cases, exercised by unit tests through the fake and guarded against real git by the existing integration suite. A `Git` struct owns the runner (`Box<dyn GitRunner>`) and exposes the operations as methods.

## Considered Options

- **Operation-level trait** (one method per operation: `status()`, `commit()`, …) — rejected: the interface is ~24 methods wide and shallow by definition, and all parsing above it goes unexercised whenever the fake is in play.
- **Stateful in-memory git simulation** as the test adapter — rejected: reimplements a meaningful slice of git (porcelain formats included), and its fidelity becomes its own bug surface. The scripted fake (arg pattern → canned output, fail on unexpected calls) is ~100 lines and makes tests read as "when git says X, git-vmr does Y".
- **Declarative operation registry/DSL** for defining git operations — rejected: the builder becomes its own interface to learn, and query operations (status, branches, worktree) don't fit it anyway.

## Consequences

- `foreach` stays outside the seam: it runs arbitrary user shell commands, not git.
- `clone` runs through the seam's interactive method; its failure propagates as an outcome instead of calling `process::exit(1)` from inside the git module. Since git streams its own stderr in interactive mode, that outcome must not print a second error line.
- The scripted fake can drift from real git output; the integration suite is deliberately kept as the second adapter's guard and must not be mass-ported onto the fake.
