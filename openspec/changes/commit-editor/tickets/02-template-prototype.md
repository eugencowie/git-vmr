# Prototype: the editor buffer template

Type: prototype
Status: open

## Question

What does the user see when the editor opens? Git shows a commented single-repo status (branch, files to be committed); we are committing across many repos with one message, so the buffer should tell the truth about all of them. Prototype a few sample buffers to react to — e.g. per-repo sections listing staged files, comment conventions, whether branch names appear — and pick one.

Use [Git editor and commit-message semantics](../research/git-editor-semantics.md) as the fidelity baseline: explicitly decide how much of Git's dynamic instructions, identity details, status, and scissors presentation the aggregate template retains. Keep presentation separate from the configurable cleanup policy settled by the seam ticket.
