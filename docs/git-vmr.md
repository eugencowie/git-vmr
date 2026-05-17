# git - the stupid content tracker

## SYNOPSIS

    git [-v | --version] [-h | --help] [-C <path>] [-c <name>=<value>]
        [--exec-path[=<path>]] [--html-path] [--man-path] [--info-path]
        [-p | --paginate | -P | --no-pager] [--no-replace-objects] [--no-lazy-fetch]
        [--no-optional-locks] [--no-advice] [--bare] [--git-dir=<path>]
        [--work-tree=<path>] [--namespace=<name>] [--config-env=<name>=<envvar>]
        <command> [<args>]

## DESCRIPTION

Git is a fast, scalable, distributed revision control system with an
unusually rich command set that provides both high-level operations and
full access to internals.

See **gittutorial**(7) to get started, then see **giteveryday**(7) for a
useful minimum set of commands. The **Git User’s Manual**\[1\] has a
more in-depth introduction.

After you mastered the basic concepts, you can come back to this page to
learn what commands Git offers. You can learn more about individual Git
commands with "git help command". **gitcli**(7) manual page gives you an
overview of the command-line command syntax.

A formatted and hyperlinked copy of the latest Git documentation can be
viewed at **https://git.github.io/htmldocs/git.html** or
**https://git-scm.com/docs**.

## OPTIONS

| Option | Description | Supported? |
| ------ | ----------- | ---------- |
| -v, --version | Prints the Git suite version that the *git* program came from. | ✅ |
| -h, --help | Prints the synopsis and a list of the most commonly used commands. If the option **--all** or **-a** is given then all available commands are printed. If a Git command is named this option will bring up the manual page for that command. | ✅ |
| -C \<path\> | Run as if git was started in *\<path\>* instead of the current working directory. When multiple **-C** options are given, each subsequent non-absolute **-C** *\<path\>* is interpreted relative to the preceding **-C** *\<path\>*. If *\<path\>* is present but empty, e.g. **-C** "", then the current working directory is left unchanged. | ✅ |
| -c \<name\>=\<value\> | Pass a configuration parameter to the command. The value given will override values from configuration files. The \<name\> is expected in the same format as listed by *git config* (subkeys separated by dots). | ❌ |
| --config-env=\<name\>=\<envvar\> | Like **-c** *\<name\>***=***\<value\>*, give configuration variable *\<name\>* a value, where \<envvar\> is the name of an environment variable from which to retrieve the value. Unlike **-c** there is no shortcut for directly setting the value to an empty string, instead the environment variable itself must be set to the empty string. It is an error if the *\<envvar\>* does not exist in the environment. *\<envvar\>* may not contain an equals sign to avoid ambiguity with *\<name\>* containing one. | ❌ |
| --exec-path\[=\<path\>\] | Path to wherever your core Git programs are installed. This can also be controlled by setting the GIT_EXEC_PATH environment variable. If no path is given, *git* will print the current setting and then exit. | ❌ |
| --html-path | Print the path, without trailing slash, where Git’s HTML documentation is installed and exit. | ❌ |
| --man-path | Print the manpath (see **man**(**1**)) for the man pages for this version of Git and exit. | ❌ |
| --info-path | Print the path where the Info files documenting this version of Git are installed and exit. | ❌ |
| -p, --paginate | Pipe all output into *less* (or if set, \$PAGER) if standard output is a terminal. This overrides the **pager.***\<cmd\>* configuration options (see the "Configuration Mechanism" section below). | ❌ |
| -P, --no-pager | Do not pipe Git output into a pager. | ❌ |
| --git-dir=\<path\> | Set the path to the repository (".git" directory). This can also be controlled by setting the **GIT_DIR** environment variable. It can be an absolute path or relative path to current working directory. | ❌ |
| --work-tree=\<path\> | Set the path to the working tree. It can be an absolute path or a path relative to the current working directory. This can also be controlled by setting the GIT_WORK_TREE environment variable and the core.worktree configuration variable (see core.worktree in **git-config**(1) for a more detailed discussion). | ❌ |
| --namespace=\<path\> | Set the Git namespace. See **gitnamespaces**(7) for more details. Equivalent to setting the **GIT_NAMESPACE** environment variable. | ❌ |
| --bare | Treat the repository as a bare repository. If GIT_DIR environment is not set, it is set to the current working directory. | ❌ |
| --no-replace-objects | Do not use replacement refs to replace Git objects. This is equivalent to exporting the **GIT_NO_REPLACE_OBJECTS** environment variable with any value. See **git-replace**(1) for more information. | ❌ |
| --no-lazy-fetch | Do not fetch missing objects from the promisor remote on demand. Useful together with **git** **cat-file** **-e** *\<object\>* to see if the object is locally available. This is equivalent to setting the **GIT_NO_LAZY_FETCH** environment variable to **1**. | ❌ |
| --no-optional-locks | Do not perform optional operations that require locks. This is equivalent to setting the **GIT_OPTIONAL_LOCKS** to **0**. | ❌ |
| --no-advice | Disable all advice hints from being printed. | ❌ |
| --literal-pathspecs | Treat pathspecs literally (i.e. no globbing, no pathspec magic). This is equivalent to setting the **GIT_LITERAL_PATHSPECS** environment variable to **1**. | ❌ |
| --glob-pathspecs | Add "glob" magic to all pathspec. This is equivalent to setting the **GIT_GLOB_PATHSPECS** environment variable to **1**. Disabling globbing on individual pathspecs can be done using pathspec magic ":(literal)" | ❌ |
| --noglob-pathspecs | Add "literal" magic to all pathspec. This is equivalent to setting the **GIT_NOGLOB_PATHSPECS** environment variable to **1**. Enabling globbing on individual pathspecs can be done using pathspec magic ":(glob)" | ❌ |
| --icase-pathspecs | Add "icase" magic to all pathspec. This is equivalent to setting the **GIT_ICASE_PATHSPECS** environment variable to **1**. | ❌ |
| --list-cmds=\<group\>\[,\<group\>...\] | List commands by group. This is an internal/experimental option and may change or be removed in the future. Supported groups are: builtins, parseopt (builtin commands that use parse-options), deprecated (deprecated builtins), main (all commands in libexec directory), others (all other commands in **\$PATH** that have git- prefix), list-\<category\> (see categories in command-list.txt), nohelpers (exclude helper commands), alias and config (retrieve command list from config variable completion.commands) | ❌ |
| --attr-source=\<tree-ish\> | Read gitattributes from \<tree-ish\> instead of the worktree. See **gitattributes**(5). This is equivalent to setting the **GIT_ATTR_SOURCE** environment variable. | ❌ |

## GIT COMMANDS

We divide Git into high level ("porcelain") commands and low level
("plumbing") commands.

## HIGH-LEVEL COMMANDS (PORCELAIN)

We separate the porcelain commands into the main commands and some
ancillary user utilities.

### Main porcelain commands

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [add](git-vmr/add.md) | Add file contents to the index. | ✅ |
| am | Apply a series of patches from a mailbox. | ❌ |
| archive | Create an archive of files from a named tree. | ❌ |
| backfill | Download missing objects in a partial clone. | ❌ |
| bisect | Use binary search to find the commit that introduced a bug. | ❌ |
| [branch](git-vmr/branch.md) | List, create, or delete branches. | ✅ |
| bundle | Move objects and refs by archive. | ❌ |
| checkout | Switch branches or restore working tree files. | ❌ |
| cherry-pick | Apply the changes introduced by some existing commits. | ❌ |
| citool | Graphical alternative to git-commit. | ❌ |
| clean | Remove untracked files from the working tree. | ❌ |
| [clone](git-vmr/clone.md) | Clone a repository into a new directory. | ✅ |
| [commit](git-vmr/commit.md) | Record changes to the repository. | ✅ |
| describe | Give an object a human readable name based on an available ref. | ❌ |
| diff | Show changes between commits, commit and working tree, etc. | ❌ |
| fetch | Download objects and refs from another repository. | ❌ |
| format-patch | Prepare patches for e-mail submission. | ❌ |
| gc | Cleanup unnecessary files and optimize the local repository. | ❌ |
| grep | Print lines matching a pattern. | ❌ |
| gui | A portable graphical interface to Git. | ❌ |
| [init](git-vmr/init.md) | Create an empty Git repository or reinitialize an existing one. | ✅ |
| log | Show commit logs. | ❌ |
| maintenance | Run tasks to optimize Git repository data. | ❌ |
| [merge](git-vmr/merge.md) | Join two or more development histories together. | ✅ |
| [mv](git-vmr/mv.md) | Move or rename a file, a directory, or a symlink. | ✅ |
| notes | Add or inspect object notes. | ❌ |
| pull | Fetch from and integrate with another repository or a local branch. | ❌ |
| push | Update remote refs along with associated objects. | ❌ |
| range-diff | Compare two commit ranges (e.g. two versions of a branch). | ❌ |
| [rebase](git-vmr/rebase.md) | Reapply commits on top of another base tip. | ✅ |
| reset | Set **HEAD** or the index to a known state. | ❌ |
| [restore](git-vmr/restore.md) | Restore working tree files. | ✅ |
| revert | Revert some existing commits. | ❌ |
| [rm](git-vmr/rm.md) | Remove files from the working tree and from the index. | ✅ |
| shortlog | Summarize *git log* output. | ❌ |
| show | Show various types of objects. | ❌ |
| sparse-checkout | Reduce your working tree to a subset of tracked files. | ❌ |
| stash | Stash the changes in a dirty working directory away. | ❌ |
| [status](git-vmr/status.md) | Show the working tree status. | ✅ |
| submodule | Initialize, update or inspect submodules. | ❌ |
| switch | Switch branches. | ❌ |
| [tag](git-vmr/tag.md) | Create, list, delete or verify tags. | ✅ |
| worktree | Manage multiple working trees. | ❌ |
| gitk | The Git repository browser. | ❌ |
| scalar | A tool for managing large Git repositories. | ❌ |

### Ancillary Commands

Manipulators:

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| config | Get and set repository or global options. | ❌ |
| fast-export | Git data exporter. | ❌ |
| fast-import | Backend for fast Git data importers. | ❌ |
| filter-branch | Rewrite branches. | ❌ |
| mergetool | Run merge conflict resolution tools to resolve merge conflicts. | ❌ |
| pack-refs | Pack heads and tags for efficient repository access. | ❌ |
| prune | Prune all unreachable objects from the object database. | ❌ |
| reflog | Manage reflog information. | ❌ |
| refs | Low-level access to refs. | ❌ |
| remote | Manage set of tracked repositories. | ❌ |
| repack | Pack unpacked objects in a repository. | ❌ |
| replace | Create, list, delete refs to replace objects. | ❌ |

Interrogators:

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| annotate | Annotate file lines with commit information. | ❌ |
| blame | Show what revision and author last modified each line of a file. | ❌ |
| bugreport | Collect information for user to file a bug report. | ❌ |
| count-objects | Count unpacked number of objects and their disk consumption. | ❌ |
| diagnose | Generate a zip archive of diagnostic information. | ❌ |
| difftool | Show changes using common diff tools. | ❌ |
| fsck | Verifies the connectivity and validity of the objects in the database. | ❌ |
| help | Display help information about Git. | ❌ |
| instaweb | Instantly browse your working repository in gitweb. | ❌ |
| merge-tree | Perform merge without touching index or working tree. | ❌ |
| rerere | Reuse recorded resolution of conflicted merges. | ❌ |
| show-branch | Show branches and their commits. | ❌ |
| verify-commit | Check the GPG signature of commits. | ❌ |
| verify-tag | Check the GPG signature of tags. | ❌ |
| version | Display version information about Git. | ❌ |
| whatchanged | Show logs with differences each commit introduces. | ❌ |
| gitweb | Git web interface (web frontend to Git repositories). | ❌ |

### Interacting with Others

These commands are to interact with foreign SCM and with other people
via patch over e-mail.

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| archimport | Import a GNU Arch repository into Git. | ❌ |
| cvsexportcommit | Export a single commit to a CVS checkout. | ❌ |
| cvsimport | Salvage your data out of another SCM people love to hate. | ❌ |
| cvsserver | A CVS server emulator for Git. | ❌ |
| imap-send | Send a collection of patches from stdin to an IMAP folder. | ❌ |
| p4 | Import from and submit to Perforce repositories. | ❌ |
| quiltimport | Applies a quilt patchset onto the current branch. | ❌ |
| request-pull | Generates a summary of pending changes. | ❌ |
| send-email | Send a collection of patches as emails. | ❌ |
| svn | Bidirectional operation between a Subversion repository and Git. | ❌ |

### Reset, restore and revert

There are three commands with similar names: **git** **reset**, **git**
**restore** and **git** **revert**.

> ·
>
> **git-revert**(1) is about making a new commit that reverts the
> changes made by other commits.

> ·
>
> **git-restore**(1) is about restoring files in the working tree from
> either the index or another commit. This command does not update your
> branch. The command can also be used to restore files in the index
> from another commit.

> ·
>
> **git-reset**(1) is about updating your branch, moving the tip in
> order to add or remove commits from the branch. This operation changes
> the commit history.
>
> **git** **reset** can also be used to restore the index, overlapping
> with **git** **restore**.
