# Research: git's editor and message-cleanup semantics

Type: research
Status: resolved

## Question

What exact behaviour would we be mirroring? Pin down, from git's own documentation and source:

- `git var GIT_EDITOR` resolution order (`GIT_EDITOR` → `core.editor` → `VISUAL` → `EDITOR` → fallback) and its failure modes — when does it error instead of answering (dumb/unset `TERM`, no editor configured)?
- How git invokes the editor (shell interpretation of the editor string, arguments, exit-code handling) and what a nonzero editor exit does to the commit.
- Default `--cleanup` behaviour for an editor-sourced message: comment-line stripping, comment character (`core.commentChar`), trailing-whitespace trimming, and when an empty message aborts the commit.
- `COMMIT_EDITMSG` conventions: where the file lives, what the commented template contains, scissors lines.
- What `git commit` does with no `-m` and no usable terminal (stdin not a TTY, dumb terminal).

## Answer

The full primary-source findings are in [Git editor and commit-message semantics](../research/git-editor-semantics.md).

Git treats editor resolution, editor execution, buffer presentation, and post-edit cleanup as separate stages. `git var GIT_EDITOR` follows `GIT_EDITOR` → `core.editor` → `VISUAL` → `EDITOR` → compiled default, except that unset/`dumb` `TERM` skips `VISUAL` and returns no value (exit 1) when `GIT_EDITOR`, `core.editor`, and `EDITOR` are all absent. Non-TTY stdin is not itself a gate: Git still attempts the resolved editor.

Editor values are shell-aware; Git appends the absolute `COMMIT_EDITMSG` path as a separate argument, treats `:` as a no-op, and aborts before committing if the editor cannot start or exits nonzero. The normal editor-sourced cleanup default is `strip`: remove comment-prefix lines, trim trailing whitespace, normalize blank lines, and abort on an empty cleaned message. The prefix and cleanup mode are configurable, and modern Git permits multi-character `core.commentString` values.

`$GIT_DIR/COMMIT_EDITMSG` is persistent enough to retain the user's text after a failed commit. Its visible contents are dynamically assembled from starting-message sources, instructions, identity details, repository status, and—when requested—a scissors marker/diff; the final cleanup contract is independent of that presentation.
