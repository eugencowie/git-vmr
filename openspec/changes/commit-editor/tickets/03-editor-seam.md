# Grilling: the interactive-spawn seam and message delivery

Type: grilling
Status: open
Blocked by: 01

## Question

`Git::output()` captures stdio, so launching an editor needs a new seam that inherits the user's terminal. Decide, with `/codebase-design`:

- Where that seam lives relative to `Git`/the runner seam (the pager precedent runs at the emit choke point; the editor must run *before* any commit).
- Where the message file lives (VMR root `.git` dir? temp?), and its lifetime.
- How the cleaned message is delivered to each child repo (`-m` vs `-F <file>`), including multi-line messages.
- Our command's abort and failure behaviour: editor nonzero exit, empty message after cleanup, no usable terminal / `git var GIT_EDITOR` failure — informed by the research ticket's facts about git's own behaviour.
