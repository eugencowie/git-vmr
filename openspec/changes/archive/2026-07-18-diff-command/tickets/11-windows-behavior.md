# Decide Windows behaviour for paging, colour, and TTY

Type: grilling
Status: resolved
Blocked by: 10

## Question

With the facts from [Windows facts for the pager and TTY decisions](10-windows-research.md) in hand, decide what `git vmr diff` promises on Windows — then amend `spec.md` (and implementation tickets 05/08 if touched):

- **Paging**: is "the `sh` spawn fails, emit silently falls back to direct printing" acceptable as the *designed* Windows behaviour, or does the pager need a Windows-aware spawn path (resolve `sh` next to git, `cmd /c`, something else)? Today that fallback is documented as untested defensive code, which Windows would hit on every paged invocation.
- **TTY under Git Bash/mintty**: if the TTY fact reports false there, Windows git users lose colour *and* paging in their most common shell. Accept, document, or handle?
- **Colour in native consoles**: anything needed for ANSI passthrough, or is it the terminal's problem?

Whatever is accepted as degradation must be written into the spec as a decision, not left as accident.

## Answer

Decided 2026-07-18 (grilling); [spec.md](../spec.md) amended ("Colour
and paging", "The pager seam", new "Windows" section, testing
requirements) and the work cut as follow-on build tickets 12–14, since
tickets 05/08 were already built:

1. **Pager spawn**: resolve the shell first-party via
   `git var GIT_SHELL_PATH`, **uniformly on all platforms** — no `cfg`
   fork, so the Linux suite exercises the exact logic Windows runs.
2. **Seam shape**: `Rendered.pager` becomes `Option<Vec<String>>`
   (argv). The diff command composes `[<shell>, "-c", <pager>]`; emit
   spawns argv verbatim, blind to shells and `-c`.
3. **Failure chain**: `GIT_SHELL_PATH` → literal `"sh"` (pre-2.45 git
   keeps today's behaviour) → spawn failure → direct print. No
   git-version × platform combination gets worse than status quo.
4. **PATH prepend**: emit prepends `dirname(argv[0])` to the pager
   child's `PATH`, mirroring git's own private-PATH augmentation, so
   Git for Windows' `less` resolves beside its `sh`. Closes the
   worst-in-matrix hole where the shell spawns but `less` is missing
   and the git-faithful no-re-print rule would lose the diff.
5. **Fallback graduates**: direct print on spawn failure is designed,
   **silent**, and unit-tested — superseding ticket 03 d3's "untested
   defensive code". It is the steady state for pre-2.45 git on native
   consoles, and the degraded output equals `--no-pager` on purpose.
6. **Colour**: Windows startup attempts to enable VT processing; on
   failure `--color=auto` resolves to `never`. VT gates auto-colour
   only — paging still gates on the TTY fact alone; explicit
   `--color=always` always passes through.
7. **TTY**: nothing to handle — `std::io::IsTerminal` recognizes
   MSYS/Cygwin PTYs, so Git Bash/mintty works. The pipe-name-convention
   caveat is an accepted safe edge; no escape-hatch flags.
8. **Enforcement**: a `windows-latest` CI test job is cut as
   [ticket 14](14-windows-ci.md), with its friendly-runner-PATH caveat
   recorded so the build effort doesn't over-claim what it proves.

Build tickets: [12-pager-shell-resolution](12-pager-shell-resolution.md),
[13-windows-vt-colour](13-windows-vt-colour.md),
[14-windows-ci](14-windows-ci.md).
