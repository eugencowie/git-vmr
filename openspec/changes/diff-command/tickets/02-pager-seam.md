# Where does the pager live?

Type: grilling
Status: open

## Question

Where does pager support sit in the codebase — a diff-local detail inside the command, or a render-layer facility other commands could later use? Decide:

- How paging composes with the existing rule that printing happens once at a single choke point (see CONTEXT.md "Rendering").
- Streaming child diffs through the pager as they arrive vs buffering the combined diff and paging once.
- How repo-outcome failure reporting interleaves with paged output: stderr vs after the pager exits, and what the pager sees.
- How pager resolution (`GIT_PAGER` → `core.pager` → `PAGER` → `less -FRX`) is tested/faked.

Consult `/codebase-design` for the seam vocabulary. The user prefers interface symmetry — mirrored type shapes over avoiding hypothetical seams.
