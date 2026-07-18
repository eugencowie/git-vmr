# Grilling: testing strategy

Type: grilling
Status: open
Blocked by: 03

## Question

How is the editor path tested at each level? Likely a scripted fake editor (a recording script set as `GIT_EDITOR`) mirroring the diff map's scripted-pager approach, plus unit tests through the seam decided in ticket 03. Decide the split between unit tests (scripted fakes, injected TTY facts) and integration tests (real editor script writing a known message, end-to-end commit across repos), and what exact assertions cover template content, cleanup, and abort paths.
