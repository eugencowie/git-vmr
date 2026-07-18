# Grilling: the interactive-spawn seam and message delivery

Type: grilling
Status: open
Blocked by: 01

## Question

`Git::output()` captures stdio, so launching an editor needs a new seam that inherits the user's terminal. Decide, with `/codebase-design`:

- Where that seam lives relative to `Git`/the runner seam (the pager precedent runs at the emit choke point; the editor must run *before* any commit).
- Where the message file lives (VMR root `.git` dir? temp?), and its lifetime.
- How the cleaned message is delivered to each child repo (`-m` vs `-F <file>`), including multi-line messages.
- The compatibility boundary for editor strings and message cleanup: arbitrary shell-interpreted editor values versus a narrower launch contract; which VMR-root Git configuration is honoured (`commit.cleanup`, `core.commentChar`/`core.commentString`); and whether modern multi-character/deprecated `auto` semantics set the version baseline.
- Our command's abort and failure behaviour: editor nonzero exit, empty message after cleanup, and `git var GIT_EDITOR` failure — preserving or intentionally improving on Git's distinction between non-TTY stdin and unset/`dumb` `TERM`, informed by [Git editor and commit-message semantics](../research/git-editor-semantics.md).
