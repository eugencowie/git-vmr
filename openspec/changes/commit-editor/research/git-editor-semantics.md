# Git editor and commit-message semantics

## Scope and source baseline

This note describes the behaviour of upstream Git at commit
[`41365c2a9ba347870b80881c0d67454edd22fd49`](https://github.com/git/git/commit/41365c2a9ba347870b80881c0d67454edd22fd49)
(the tip of `git/git`'s `master` branch when researched). The user-facing
contracts are taken from Git's own documentation; implementation details and
edge cases are pinned to that source revision.

## `git var GIT_EDITOR`

The documented preference order is:

1. the `GIT_EDITOR` environment variable;
2. the `core.editor` configuration value;
3. the `VISUAL` environment variable;
4. the `EDITOR` environment variable;
5. the build-time default, usually `vi`.

Git documents both this order and that the resulting value is intended for
shell interpretation ([`git-var` documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-var.adoc#L41-L50)).

There is one important qualification in the implementation. Git defines a
terminal as "dumb" when `TERM` is either unset or exactly `dumb`. In that case
it deliberately skips `VISUAL`. It still accepts, in order, `GIT_EDITOR`,
`core.editor`, and `EDITOR`; but if none of those has a value it returns no
editor instead of using the compiled fallback. With a non-dumb `TERM`, the
ordinary order applies and the compiled fallback is used when nothing is
configured ([`editor.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/editor.c#L17-L45)). Thus:

- **No editor configured, non-dumb `TERM`:** `git var GIT_EDITOR` prints the
  compiled default and succeeds.
- **`TERM` unset or `dumb`, but `EDITOR` configured:** it prints `EDITOR` and
  succeeds.
- **`TERM` unset or `dumb`, only `VISUAL` configured:** `VISUAL` is ignored;
  the command has no value.
- **`TERM` unset or `dumb`, and no `GIT_EDITOR`/`core.editor`/`EDITOR`:**
  `git var GIT_EDITOR` prints nothing and exits with status 1. It does not emit
  the interactive editor error itself. `git var`'s documented no-value result
  is exit 1, and its implementation returns 1 when the reader returns `NULL`
  ([documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-var.adoc#L14-L17),
  [`builtin/var.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/var.c#L217-L247)).

Git's own editor test covers the two unusual dumb-terminal rules: falling back
to `vi` must fail, and `EDITOR` must beat `VISUAL`
([`t/t7005-editor.sh`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/t/t7005-editor.sh#L37-L45)).

## How Git invokes the editor

For a real editor command, Git resolves `COMMIT_EDITMSG` to a real, absolute
path, supplies that path as the argument after the editor command, and marks
the child as `use_shell`. The literal editor value `:` is a special no-op: Git
does not start a child and treats the edit as successful
([`editor.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/editor.c#L60-L124)).

`use_shell` does not mean Git blindly concatenates an unquoted filename. Its
command runner examines the editor string. A simple executable name is run
directly with the message path as its next argument. If the string contains
shell-special characters (including whitespace, quotes, variables, `~`,
redirections, or operators), Git runs its configured POSIX shell with `-c` and
appends the separately supplied arguments through `"$@"`
([`run-command.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/run-command.c#L288-L308)). This is why values such as `vim --nofork`, quoted
paths, environment-variable references, and even redirections are valid editor
strings. Git's tests explicitly exercise an editor path containing an escaped
space ([`t/t7005-editor.sh`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/t/t7005-editor.sh#L89-L99)).

Failure handling is strict:

- a missing editor in dumb-terminal mode produces `Terminal is dumb, but
  EDITOR unset`;
- failure to spawn produces `unable to start editor '<value>'`;
- any nonzero child result produces `there was a problem with the editor
  '<value>'`;
- `SIGINT` or `SIGQUIT` from the editor is re-raised by Git.

These paths are implemented in
[`launch_specified_editor`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/editor.c#L60-L117). In `git commit`, any editor-launch failure is followed by
`Please supply the message using either -m or -F option.` and an immediate exit
with status 1. The `commit-msg` hook and commit creation have not happened at
that point ([`builtin/commit.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L1116-L1135)). The prepared/edited
`COMMIT_EDITMSG` file remains available after this error.

## Default cleanup for an editor-sourced message

Absent `--cleanup` or `commit.cleanup`, Git maps `default` to `strip` when an
editor is used and to `whitespace` otherwise. Explicit modes map as follows:
`verbatim` means no cleanup; `whitespace` means whitespace cleanup; `strip`
means whitespace plus comment removal; `scissors` means scissors cleanup when
editing but degrades to `whitespace` when no editor is used
([`sequencer.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/sequencer.c#L684-L700)). `commit.cleanup` can override the default
([`commit` configuration documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/config/commit.adoc#L7-L15)).

For the normal no-`-m` editor flow, the resulting default `strip` behaviour is:

- remove every line that **begins** with the configured comment prefix;
- remove whitespace at the end of every line;
- remove leading and trailing blank/whitespace-only lines;
- collapse consecutive interior blank lines to one;
- ensure a nonempty final line ends in a newline.

The high-level contract is in the
[`--cleanup` documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-commit.adoc#L235-L261); the exact whitespace algorithm and
start-of-line prefix test are in
[`strbuf_stripspace`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/strbuf.c#L1063-L1128). A comment prefix appearing after indentation is not a
comment for stripping purposes.

The default comment prefix is `#`. Modern Git treats `core.commentChar` and
`core.commentString` as aliases and permits a multi-character string; older
Git before 2.45 only accepted a single ASCII byte for `commentChar`. Lines
beginning with the selected prefix are removed after the editor returns
([`core.commentChar` documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/config/core.adoc#L540-L578)). The same prefix is used to write
Git's instructional and status lines, so changing it changes both template
generation and cleanup.

After the editor and `commit-msg` hook, Git rereads `COMMIT_EDITMSG`, performs
cleanup, and checks the result. Unless `--allow-empty-message` is present, a
cleaned message containing only whitespace (or only `Signed-off-by:` lines)
aborts with `Aborting commit due to empty commit message.`
([empty check](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/sequencer.c#L1186-L1233),
[`git commit` abort](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L1896-L1915)). An unchanged nonempty custom template is a
separate abort condition: `Aborting commit; you did not edit the message.` The
documented escape hatch is `--allow-empty-message`
([`git-commit` documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-commit.adoc#L230-L233)).

## `COMMIT_EDITMSG` and the editor template

The file is `$GIT_DIR/COMMIT_EDITMSG`; Git's path helper constructs precisely
that Git-directory-relative name
([`sequencer.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/sequencer.c#L63-L68)). Git documents it as the message of the commit in progress:
on a pre-commit-creation error, the user's message remains there until the next
`git commit` invocation overwrites it
([`git-commit` documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-commit.adoc#L575-L583)).

The editor buffer is assembled rather than being one fixed template. Git first
writes any starting message (for example a configured `commit.template`, merge
message, squash message, or reused commit message). For the ordinary editor
flow with status enabled, it then adds:

- a blank separator;
- commented instructions explaining whether comment-prefixed lines are
  removed or kept and whether an empty message aborts;
- when relevant, commented author, author date, and committer identity lines;
- a commented long-form status describing the branch and staged, unstaged,
  unmerged, and untracked state.

The construction, including conditional identity lines and the status call,
is in
[`prepare_to_commit`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L911-L1027). Status inclusion is controlled by
`--[no-]status`/`commit.status`, whose default is true
([configuration documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/config/commit.adoc#L24-L31)). Git intentionally forces comment
prefixes in `COMMIT_EDITMSG` even if `status.displayCommentPrefix=false`
([`builtin/commit.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L911-L926)).

The canonical scissors marker is:

```text
# ------------------------ >8 ------------------------
```

where `#` is replaced by the configured comment prefix. In scissors mode,
everything from the matching marker line onward is discarded. The match must
start a line and uses the configured prefix followed by one space and the
fixed cut text
([documentation](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/Documentation/git-commit.adoc#L248-L257),
[`wt_status_locate_end`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/wt-status.c#L1124-L1138)). When Git adds a scissors line itself, it follows it
with commented text warning that everything below is ignored
([`wt-status.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/wt-status.c#L1141-L1158)). `git commit -v` also places its un-commented diff
below a scissors line; final cleanup truncates before that diff even under the
normal editor cleanup mode
([diff placement](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/wt-status.c#L1182-L1203),
[`cleanup_message`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/sequencer.c#L1212-L1221)).

## No `-m` and no usable terminal

For ordinary `git commit` with no message-giving option, `use_editor` starts as
true. It is disabled by `-m`, `-F`, or `-C`-style sources (unless `--edit`
turns it back on), but there is **no `isatty(stdin)` check** in this decision
([initial value](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L135-L147),
[option handling](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L1309-L1414)). Therefore "stdin is not a TTY" and "`TERM` is dumb" are distinct cases:

- **Non-TTY stdin by itself:** Git still prepares `COMMIT_EDITMSG` and launches
  the resolved editor. A configured noninteractive editor can succeed. A
  terminal editor may fail or behave poorly because of its own terminal
  requirements; Git treats that like any other nonzero editor result. Git does
  not proactively replace this with a "use `-m`" error.
- **`TERM` unset/dumb and no usable explicit editor:** editor resolution returns
  no value, launch emits `Terminal is dumb, but EDITOR unset`, and `git commit`
  follows with `Please supply the message using either -m or -F option.` before
  exiting 1. This holds whether or not stdin is a TTY.
- **`TERM` unset/dumb with `GIT_EDITOR`, `core.editor`, or `EDITOR`:** Git still
  launches that editor; dumb `TERM` is not itself a ban on editing. `VISUAL`
  alone does not count in this mode.

The only stdin-TTY check in message acquisition is for the separate `-F -`
case, where it merely prints a notice before reading the message from stdin
([`builtin/commit.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/builtin/commit.c#L804-L817)). Editor launch instead uses `isatty(stderr)` only
to decide whether to print the optional "Waiting for your editor" advice
([`editor.c`](https://github.com/git/git/blob/41365c2a9ba347870b80881c0d67454edd22fd49/editor.c#L66-L85)).

## Implications for mirroring

A faithful mirror needs to separate four concerns that are easy to collapse:

1. editor **resolution**, including the dumb-terminal exception and silent
   no-value result of `git var`;
2. editor **execution**, including shell-aware command strings, a separately
   appended absolute file path, `:` as a no-op, and strict exit handling;
3. editor-buffer **presentation**, which is state/configuration-dependent and
   uses the active comment prefix;
4. post-edit **cleanup and validation**, including scissors/diff truncation,
   prefix stripping, whitespace normalization, and empty/untouched-template
   aborts.

In particular, a mirror should not use `isatty(stdin)` as a substitute for
Git's `TERM`/editor-resolution rules, and it should not infer cleanup semantics
only from the visible commented template: the final cleanup mode is chosen
independently and may be configured to keep those lines.
