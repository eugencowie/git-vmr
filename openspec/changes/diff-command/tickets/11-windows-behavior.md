# Decide Windows behaviour for paging, colour, and TTY

Type: grilling
Blocked by: 10

## Question

With the facts from [Windows facts for the pager and TTY decisions](10-windows-research.md) in hand, decide what `git vmr diff` promises on Windows — then amend `spec.md` (and implementation tickets 05/08 if touched):

- **Paging**: is "the `sh` spawn fails, emit silently falls back to direct printing" acceptable as the *designed* Windows behaviour, or does the pager need a Windows-aware spawn path (resolve `sh` next to git, `cmd /c`, something else)? Today that fallback is documented as untested defensive code, which Windows would hit on every paged invocation.
- **TTY under Git Bash/mintty**: if the TTY fact reports false there, Windows git users lose colour *and* paging in their most common shell. Accept, document, or handle?
- **Colour in native consoles**: anything needed for ANSI passthrough, or is it the terminal's problem?

Whatever is accepted as degradation must be written into the spec as a decision, not left as accident.
