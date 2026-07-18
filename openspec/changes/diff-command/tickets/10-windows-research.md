# Windows facts for the pager and TTY decisions

Type: research
Status: ready-for-agent

## Question

The spec commits to `sh -c` pager spawning and an injected TTY bool, both settled on POSIX assumptions, yet `x86_64-pc-windows-msvc` is a shipped release target. Surface the facts the Windows decision waits on, against primary sources (git source/docs, Rust std source, real tools):

- How does `git.exe` itself spawn the pager on native Windows? (It pages fine from cmd/PowerShell — via what shell, found how?)
- For an *external* process (a Rust binary run from cmd/PowerShell with a default Git for Windows install), is `sh` on PATH? Is `less`? What does `git var GIT_PAGER` return there?
- Is there a reliable way for a non-MSYS process to reach Git for Windows' bundled `sh`/`less` (e.g. resolving relative to `git --exec-path` or the git.exe location), and do any real tools do this?
- What do comparable Rust/Go CLIs that page (`gh`, `delta`, `bat`) do on native Windows — spawn via what, fall back to what?
- Does Rust std's `IsTerminal` return true under Git Bash/mintty (MSYS/Cygwin pty pipes), or only in native consoles? (The old atty crate said false; `is-terminal`/std may have added the msys pty-name check — verify which behaviour current std has.)
- If we pass git's `--color=always` ANSI output straight through to a native console, does it render? (Windows Terminal vs legacy conhost / ENABLE_VIRTUAL_TERMINAL_PROCESSING.)

Capture findings as `openspec/changes/diff-command/research/windows-pager.md`; link it here.
