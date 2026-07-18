# Windows pager and TTY facts

Research date: 2026-07-18. Source links below point to the upstream project or vendor that owns the behavior. Git facts distinguish upstream Git's generic pager machinery from the Git for Windows (`GIT_WINDOWS_NATIVE`) startup and process layer.

## Decision summary

- `git.exe` does **not** unconditionally run a pager through `sh -c`. Upstream Git marks the pager as `use_shell`, but the command runner invokes a shell only when the configured pager string contains shell-special characters (including whitespace). The default `less` is therefore launched directly. A pager such as `less -R` is run as `<resolved sh> -c 'less -R'`.
- A normal Git for Windows installation can do this from `cmd.exe` or PowerShell because `git.exe` privately prepends its own `mingw64\bin` and `usr\bin` directories to **its process's** `PATH`. The recommended installer option adds only `Git\cmd` to the user's persistent `PATH`; consequently, an unrelated native Rust process normally cannot resolve bare `sh` or `less`, even though Git can.
- `git var GIT_PAGER` reports a command string, usually `less`; it does not report an executable path. `git var GIT_SHELL_PATH` is the useful first-party locator: current Git for Windows resolves it to the `sh` it would use. Treat deriving `less.exe` as a sibling of that path, or walking upward from `git --exec-path`, as a Git-for-Windows-layout heuristic rather than a Git API.
- The current `gh`, `delta`, and `bat` implementations do not borrow Git for Windows' shell or pager discovery. They split a configured pager into argv, resolve the executable on the caller's `PATH`, and spawn it directly. `delta` and `bat` fall back to stdout when the pager is absent; `gh` has no default pager and its pager start method returns an error when a configured executable cannot be found.
- Current Rust `std::io::IsTerminal` does recognize MSYS/Cygwin pseudo-terminals. On Windows it first calls `GetConsoleMode`; if that fails, it recognizes a pipe whose final kernel name starts with `msys-` or `cygwin-` and contains `-pty`. Thus Git Bash/mintty is not limited by the old native-console-only behavior, subject to the handle actually having the expected MSYS/Cygwin pipe name.
- Passing ANSI bytes through unchanged is not universally safe on Windows. Windows console VT parsing is conditional on `ENABLE_VIRTUAL_TERMINAL_PROCESSING`. Windows Terminal and Windows 10+ Console Host are VT-capable, but code writing via a console handle must not assume that the mode is enabled; older Console Host cannot provide this facility. The relay should enable VT mode where possible, use a translation layer, or disable/strip forced color when neither is available.

## How Git for Windows starts its pager

Upstream Git chooses the pager in this order: `GIT_PAGER`, `core.pager`, `PAGER`, then the compile-time default, which is `less` unless overridden. An empty value or `cat` disables paging. `prepare_pager_args` stores the entire pager string as the first argument and sets `use_shell = 1`; it also supplies Git's usual default `LESS=FRX` and `LV=-c` environment values when the user has not set them. [Git for Windows `pager.c`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/pager.c#L45-L111) and the [upstream build defaults](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/Makefile#L2429-L2436) own these facts.

`use_shell` is conditional in effect. Git's command runner scans the first argument for shell-special characters. Only if it finds one does it prepend `git_shell_path()` and `-c`; otherwise it directly resolves and executes the pager program. Whitespace is in that special-character set, so the practical cases are:

```text
default "less"       -> resolve and execute less directly
configured "less -R" -> <git_shell_path()> -c "less -R"
```

This is visible in [`prepare_shell_cmd`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/run-command.c#L417-L435) and [`prepare_cmd`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/run-command.c#L992-L1019). On non-Windows builds `git_shell_path()` returns the compiled `SHELL_PATH`; under `GIT_WINDOWS_NATIVE` it instead calls Git's Windows `locate_in_PATH("sh")` and normalizes the slashes. [Git for Windows `run-command.c`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/run-command.c#L408-L415).

Git for Windows makes that lookup succeed without requiring `sh` in the calling `cmd.exe` environment. During native startup, when `MSYSTEM` is absent, it derives the installation prefix from the running executable and prepends `<root>\<mingw-prefix>\bin` and `<root>\usr\bin` to its own `PATH`, after an optional user `bin`. [The prefix derivation and directory construction](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/compat/mingw.c#L3694-L3729) and [the process-local environment update](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/compat/mingw.c#L3812-L3832) are Git-for-Windows-specific code. The native lookup then walks the semicolon-separated `PATH`, applying Windows executable rules, before using `CreateProcessW` through the MinGW compatibility layer. [Git for Windows `path_lookup`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/compat/mingw.c#L1989-L2026) and [`mingw_spawnvpe`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/compat/mingw.c#L2432-L2456).

Therefore the answer to “which shell?” is: no shell for a simple default pager name; otherwise the `sh.exe` found on Git's augmented process-local `PATH`, normally the Git for Windows copy under `usr\bin`.

## What an unrelated native process sees

The Git for Windows installer has three PATH choices. The default/recommended `Cmd` choice says it adds only minimal Git wrappers and deliberately avoids the optional Unix tools. The broader `CmdTools` choice exposes both Git and the optional Unix tools. [Installer UI and default selection](https://github.com/git-for-windows/build-extra/blob/3f5b0672b8c9050a28c03c61fa3a45914465bc86/installer/install.iss#L2220-L2242).

The implementation matches the wording: both choices add `<app>\cmd`, but only `CmdTools` also adds `<app>\<mingw-bitness>\bin` and `<app>\usr\bin`. [Installer PATH mutation](https://github.com/git-for-windows/build-extra/blob/3f5b0672b8c9050a28c03c61fa3a45914465bc86/installer/install.iss#L3412-L3440). Consequently, with the normal recommended installation:

- `git.exe` is on the persistent PATH via `Git\cmd`.
- Bare `sh` and `less` are normally **not** on the persistent PATH seen by an unrelated Rust binary launched from `cmd.exe` or PowerShell.
- Git itself can still find its bundled tools because it changes its own environment during startup, as described above.
- With the non-default `CmdTools` installer choice, the unrelated process can normally resolve `sh.exe` and `less.exe` from `usr\bin`.

The exact installed file set is distribution-dependent: this conclusion applies to the normal full Git for Windows installer discussed by the ticket, not necessarily MinGit or a manually pruned/portable layout. The installer source establishes the PATH behavior, but this research did not independently inventory every current release flavor's `less.exe` payload.

`git var GIT_PAGER` does not bridge the visibility gap. Its implementation calls the same pager-selection routine with TTY assumed true and prints `cat` only when pagination is disabled; with no overrides it therefore prints the compile-time default `less`. [Git `builtin/var.c`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/builtin/var.c#L44-L51). Git's documentation explicitly defines the value as a shell-interpreted command and gives the preference order; it does not promise an executable path. [Git `git-var` documentation](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/Documentation/git-var.adoc#L60-L67).

## Reaching the bundled tools reliably

Current Git exposes `GIT_SHELL_PATH` through `git var`. The documentation calls it “the path of the binary providing the POSIX shell,” and the implementation returns `git_shell_path()`. [Documentation](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/Documentation/git-var.adoc#L72-L74) and [implementation](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/builtin/var.c#L58-L61). On current Git for Windows that is the strongest first-party answer for locating the shell Git itself would use:

```text
git var GIT_SHELL_PATH
```

An external process can execute that returned path directly with `-c`. This avoids assuming that bare `sh` is on the parent's PATH. It does require a sufficiently recent Git version that implements the variable; callers supporting older Git need an explicit fallback.

There is no parallel `GIT_PAGER_PATH`: `GIT_PAGER` is intentionally a command string. In the normal Git for Windows layout, `less.exe` is expected beside `sh.exe` under `usr\bin`, so taking the parent directory of `GIT_SHELL_PATH` and checking for `less.exe` is a practical, testable fallback. It is **not** guaranteed by upstream Git's `git-var` contract. Likewise, `git --exec-path` identifies Git's subprogram directory (normally below `mingw64\libexec\git-core`), not the Unix-tool directory; walking from it to `usr\bin` bakes in more layout knowledge. Git's exec-path implementation resolves ancillary Git tooling relative to the executable when built with `RUNTIME_PREFIX`. [Git `exec-cmd.c`](https://github.com/git-for-windows/git/blob/053e0828c772cb78affee38352b843ebdca78cbc/exec-cmd.c#L301-L347).

Recommended reliability order for a native caller needing Git's semantics is therefore:

1. Accept an explicit pager configuration/path from the user.
2. For shell interpretation, query and execute `git var GIT_SHELL_PATH` directly.
3. For the default `less`, resolve normally on the caller's PATH; if absent, optionally probe the checked sibling of `GIT_SHELL_PATH` and treat failure as “no pager.”
4. Do not treat the output `less` from `git var GIT_PAGER` as evidence that `less` is resolvable by the caller.

No examined peer tool implements the Git-for-Windows-specific sibling or exec-path discovery; all use ordinary process resolution, below.

## Comparable CLI behavior

### GitHub CLI (`gh`, Go)

`gh`'s pager precedence is `GH_PAGER`, configured `pager`, then `PAGER`; there is no implicit `less` default. [Factory configuration](https://github.com/cli/cli/blob/2af8c115be240a8018add33bf5c7a9ba5070a62c/pkg/cmd/factory/default.go#L151-L160). Its I/O layer skips paging for an empty value, `cat`, or non-TTY output. Otherwise it parses the command with Go shlex, resolves argv[0] with `safeexec.LookPath`, and calls `exec.Command` directly—no `cmd.exe`, PowerShell, or `sh -c`. If resolution or start fails, `StartPager` returns the error; it has no internal stdout fallback after a non-empty pager was requested. [`gh` `StartPager`](https://github.com/cli/cli/blob/2af8c115be240a8018add33bf5c7a9ba5070a62c/pkg/iostreams/iostreams.go#L205-L249).

Practical native-Windows result: with no pager environment/configuration, `gh` simply does not page. Setting `GH_PAGER=less` under the recommended Git for Windows PATH does not make Git's private `usr\bin` visible to `gh`.

### `delta` (Rust)

`delta` defaults to `less`, with `DELTA_PAGER`/`PAGER` and command-line/config overrides. It parses the command with `shell_words`, resolves the binary with `grep_cli::resolve_binary`, and launches it directly with `std::process::Command`. If resolution or spawning fails, it falls back to stdout. [`delta` pager implementation](https://github.com/dandavison/delta/blob/f85c46ba8b913aa3208af0f3573db90286e56e18/src/utils/bat/output.rs#L79-L155) and [direct process construction](https://github.com/dandavison/delta/blob/f85c46ba8b913aa3208af0f3573db90286e56e18/src/utils/bat/output.rs#L174-L255).

Thus native `delta` does not use Git's shell or augment its PATH; under a recommended Git for Windows install with no separately visible `less`, its automatic paging degrades to stdout.

### `bat` (Rust)

`bat` also selects explicit config, `BAT_PAGER`, `PAGER`, then default `less`; `PAGER=more`, `most`, or `bat` is normalized to `less` to preserve color and avoid recursion. [`bat` pager selection](https://github.com/sharkdp/bat/blob/78951393e29bfd2f2a45f4326b9d2bb5e737dd2a/src/pager.rs#L99-L137). It resolves an external pager and direct-spawns it with `Command`; an unresolvable or failed pager produces a warning and stdout fallback. It also supports an in-process `minus` pager when explicitly configured as `builtin`, but `builtin` is not the default. [`bat` output implementation](https://github.com/sharkdp/bat/blob/78951393e29bfd2f2a45f4326b9d2bb5e737dd2a/src/output.rs#L91-L199).

The consistent peer-tool precedent is argv parsing plus direct spawn, with stdout as the safe degradation (except `gh`, whose normal unset state avoids paging and whose configured-start failure is returned).

## Rust `IsTerminal` under Git Bash/mintty

Current Rust standard library behavior is not native-console-only. On Windows, `is_terminal` first checks `GetConsoleMode`; success is definitive for a native console handle. On failure it calls an MSYS fallback. That fallback requires `GetFileType(handle) == FILE_TYPE_PIPE`, queries `FileNameInfo` with `GetFileInformationByHandleEx`, takes the last path component, and returns true exactly when the name starts with `msys-` or `cygwin-` and contains `-pty`. [Current Rust std source](https://github.com/rust-lang/rust/blob/b26c8ef0535b2de24a0af4048370caf449eebabd/library/std/src/sys/io/is_terminal/windows.rs#L5-L69).

Therefore `stdout().is_terminal()`/`stderr().is_terminal()` should return true for the conventional MSYS/Cygwin PTY pipes used by Git Bash/mintty, even though `GetConsoleMode` alone would fail. It returns false for an ordinary redirected pipe. The remaining caveat is structural: detection depends on the kernel pipe name matching that convention; an unusual terminal bridge or future naming change can still produce false. This finding is from current Rust source, not the historical `atty` crate behavior.

## Relaying `--color=always` to Windows output

Microsoft documents that the Windows console host intercepts ANSI/VT output sequences only when `ENABLE_VIRTUAL_TERMINAL_PROCESSING` is set on the output screen-buffer handle with `SetConsoleMode`. The documented example explicitly reads the current mode, ORs in that flag, and writes SGR sequences afterward. [Console Virtual Terminal Sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences). The mode reference defines the flag as enabling VT100-style cursor and color/font parsing and notes that `ENABLE_PROCESSED_OUTPUT` must also be enabled. [High-Level Console Modes](https://learn.microsoft.com/en-us/windows/console/high-level-console-modes).

Microsoft separately describes both Windows 10+ Console Host and Windows Terminal as xterm-compatible. [PowerShell ANSI terminal documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_ansi_terminals?view=powershell-7.6). That is capability, not a guarantee about the mode inherited by an arbitrary native process. A Rust program that captures `git --color=always` through a pipe and copies the bytes to a Windows console handle has moved responsibility for final rendering to its own output path:

- With VT processing enabled, SGR color sequences render in modern Console Host/Windows Terminal.
- Without that mode, raw pass-through is not guaranteed to render correctly, even if Windows Terminal is the visible frontend.
- Pre-VT legacy Console Host cannot be repaired merely by passing the bytes through; the application needs a Win32 color translation layer or uncolored output.
- If stdout is an MSYS/mintty PTY pipe, the terminal consumes VT bytes on the other side; enabling a Windows console mode is neither applicable nor necessary for that pipe handle.

The robust design is to attempt `GetConsoleMode`/`SetConsoleMode(ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT)` for a native console, preserve raw ANSI for a recognized MSYS/Cygwin PTY, and otherwise avoid promising rendered color. Merely injecting an `IsTerminal` boolean answers whether paging is appropriate; it does not by itself establish that ANSI pass-through is renderable.

## Implications for the pending decisions

1. A universal Windows implementation of literal `sh -c <pager>` cannot assume `sh` is on PATH. If Git-compatible shell semantics are required, use `git var GIT_SHELL_PATH` and execute that absolute path; otherwise follow peer tools and parse/direct-spawn the pager.
2. Treat `git var GIT_PAGER` as configuration text, not discovery. A default value of `less` can be valid for Git and invalid for the caller at the same time.
3. Missing pager should degrade to direct stdout. This matches `delta` and `bat` and handles the recommended Git for Windows PATH cleanly.
4. The injected TTY bool can be backed by current Rust `IsTerminal` on Windows, including normal Git Bash/mintty. Tests should still inject both true and false rather than depending on the host terminal.
5. Color policy needs a separate output-capability decision. TTY detection and ANSI support are related but not equivalent on native Windows.
