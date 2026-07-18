# Windows facts for the pager and TTY decisions

Type: research
Status: resolved

## Question

The spec commits to `sh -c` pager spawning and an injected TTY bool, both settled on POSIX assumptions, yet `x86_64-pc-windows-msvc` is a shipped release target. Surface the facts the Windows decision waits on, against primary sources (git source/docs, Rust std source, real tools):

- How does `git.exe` itself spawn the pager on native Windows? (It pages fine from cmd/PowerShell — via what shell, found how?)
- For an *external* process (a Rust binary run from cmd/PowerShell with a default Git for Windows install), is `sh` on PATH? Is `less`? What does `git var GIT_PAGER` return there?
- Is there a reliable way for a non-MSYS process to reach Git for Windows' bundled `sh`/`less` (e.g. resolving relative to `git --exec-path` or the git.exe location), and do any real tools do this?
- What do comparable Rust/Go CLIs that page (`gh`, `delta`, `bat`) do on native Windows — spawn via what, fall back to what?
- Does Rust std's `IsTerminal` return true under Git Bash/mintty (MSYS/Cygwin pty pipes), or only in native consoles? (The old atty crate said false; `is-terminal`/std may have added the msys pty-name check — verify which behaviour current std has.)
- If we pass git's `--color=always` ANSI output straight through to a native console, does it render? (Windows Terminal vs legacy conhost / ENABLE_VIRTUAL_TERMINAL_PROCESSING.)

Capture findings as `openspec/changes/diff-command/research/windows-pager.md`; link it here.

## Answer

Research captured in [Windows pager and TTY facts](../research/windows-pager.md).

The decision-relevant facts are:

- Git for Windows makes its pager work from native shells by privately
  augmenting `git.exe`'s process `PATH`; the recommended installer PATH
  exposes `Git\cmd`, not the bundled `sh` or `less`, to an unrelated
  native process.
- `git var GIT_PAGER` returns command text, usually `less`, not a usable
  executable path. Current Git provides `git var GIT_SHELL_PATH` as the
  first-party way to locate the shell Git itself would use; deriving a
  sibling `less.exe` remains a checked Git-for-Windows-layout heuristic.
- Git direct-spawns a simple pager executable and invokes its shell only
  for command strings containing shell-special characters. `gh`, `delta`,
  and `bat` likewise direct-spawn parsed pager argv rather than borrowing
  Git's shell discovery; `delta` and `bat` fall back to stdout when the
  pager is unavailable.
- Current Rust `std::io::IsTerminal` recognizes conventional MSYS/Cygwin
  PTY pipe names, so normal Git Bash/mintty output is not limited to the
  old native-console-only behavior.
- ANSI pass-through and TTY detection are distinct on native Windows:
  modern Windows terminals can render VT sequences, but a console handle
  requires VT processing to be enabled; MSYS/mintty PTY pipes consume the
  sequences on the terminal side.

The artifact cites the owning Git, Git for Windows, Rust, peer-tool, and
Microsoft sources and calls out the remaining distribution-layout caveat.
