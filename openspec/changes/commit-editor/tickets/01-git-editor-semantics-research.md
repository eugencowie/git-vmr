# Research: git's editor and message-cleanup semantics

Type: research
Status: open

## Question

What exact behaviour would we be mirroring? Pin down, from git's own documentation and source:

- `git var GIT_EDITOR` resolution order (`GIT_EDITOR` → `core.editor` → `VISUAL` → `EDITOR` → fallback) and its failure modes — when does it error instead of answering (dumb/unset `TERM`, no editor configured)?
- How git invokes the editor (shell interpretation of the editor string, arguments, exit-code handling) and what a nonzero editor exit does to the commit.
- Default `--cleanup` behaviour for an editor-sourced message: comment-line stripping, comment character (`core.commentChar`), trailing-whitespace trimming, and when an empty message aborts the commit.
- `COMMIT_EDITMSG` conventions: where the file lives, what the commented template contains, scissors lines.
- What `git commit` does with no `-m` and no usable terminal (stdin not a TTY, dumb terminal).
