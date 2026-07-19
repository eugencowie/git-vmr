# Prototype: the editor buffer template

Type: prototype
Status: resolved

## Question

What does the user see when the editor opens? Git shows a commented single-repo status (branch, files to be committed); we are committing across many repos with one message, so the buffer should tell the truth about all of them. Prototype a few sample buffers to react to — e.g. per-repo sections listing staged files, comment conventions, whether branch names appear — and pick one.

Use [Git editor and commit-message semantics](../research/git-editor-semantics.md) as the fidelity baseline: explicitly decide how much of Git's dynamic instructions, identity details, status, and scissors presentation the aggregate template retains. Keep presentation separate from the configurable cleanup policy settled by the seam ticket.

## Answer

Use the aggregate Git-style status explored as Variant B:

- Start with dynamic instructions saying that one message creates the number
  of child commits about to be made, comment-prefixed lines are ignored, and
  an empty message aborts every commit.
- Group repositories by branch, with `main` first and the remaining branch
  groups in stable order. Name the repositories in each `On branch …`
  heading.
- Within every branch group, render Git-style `Changes to be committed:` and
  `Changes not staged for commit:` sections. Use Git's long status labels and
  indentation (`modified:`, `new file:`, `deleted:`, and so on), but make every
  path VMR-root-relative by prepending its repository name.
- Separate branch groups with an extra commented blank line so independent
  groups remain visually distinct in a large buffer.
- Do not show identity details, untracked files, or a scissors block. The
  active comment prefix and cleanup behaviour remain independent policy owned
  by [Grilling: the interactive-spawn seam and message delivery](03-editor-seam.md).

The ordinary single-repository Git buffer was retained as a comparison but
rejected because it necessarily hides the other child repositories.

Prototype source: [`prototype/commit-editor-template` at `a5e00af`](../../../../.git/refs/heads/prototype/commit-editor-template).
Inspect it with `git show prototype/commit-editor-template:openspec/changes/commit-editor/prototypes/template-buffer/README.md`.
