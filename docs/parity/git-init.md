# git-init - Create an empty Git repository or reinitialize an existing one

## SYNOPSIS

    git init [-q | --quiet] [--bare] [--template=<template-directory>]
             [--separate-git-dir <git-dir>] [--object-format=<format>]
             [--ref-format=<format>]
             [-b <branch-name> | --initial-branch=<branch-name>]
             [--shared[=<permissions>]] [<directory>]

## DESCRIPTION

This command creates an empty Git repository - basically a **.git**
directory with subdirectories for **objects**, **refs/heads**,
**refs/tags**, and template files. An initial branch without any commits
will be created (see the **--initial-branch** option below for its
name).

If the **GIT_DIR** environment variable is set then it specifies a path
to use instead of **./.git** for the base of the repository.

If the object storage directory is specified via the
**GIT_OBJECT_DIRECTORY** environment variable then the sha1 directories
are created underneath; otherwise, the default **\$GIT_DIR/objects**
directory is used.

Running **git** **init** in an existing repository is safe. It will not
overwrite things that are already there. The primary reason for
rerunning **git** **init** is to pick up newly added templates (or to
move the repository to another place if **--separate-git-dir** is
given).

## OPTIONS

| Option | Description | Supported? |
| ------ | ----------- | ---------- |
| **-q**, **--quiet** | Only print error and warning messages; all other output will be suppressed. | ❌ |
| **--bare** | Create a bare repository. If **GIT_DIR** environment is not set, it is set to the current working directory. | ❌ |
| **--object-format=***\<format\>* | Specify the given object *\<format\>* (hash algorithm) for the repository. The valid values are **sha1** and (if enabled) **sha256**. **sha1** is the default. | ❌ |
| **--ref-format=***\<format\>* | Specify the given ref storage *\<format\>* for the repository. The valid values are: | ❌ |
| **--template=***\<template-directory\>* | Specify the directory from which templates will be used. (See the "TEMPLATE DIRECTORY" section below.) | ❌ |
| **--separate-git-dir=***\<git-dir\>* | Instead of initializing the repository as a directory to either **\$GIT_DIR** or **./.git/**, create a text file there containing the path to the actual repository. This file acts as a filesystem-agnostic Git symbolic link to the repository. | ❌ |
| **-b** *\<branch-name\>*, **--initial-branch=***\<branch-name\>* | Use *\<branch-name\>* for the initial branch in the newly created repository. If not specified, fall back to the default name (currently **master**, but this will change to **main** when Git 3.0 is released). The default name can be customized via the **init.defaultBranch** configuration variable. | ❌ |
| **--shared**\[**=**(**false**\\|**true**\\|**umask**\\|**group**\\|**all**\\|**world**\\|**everybody**\\|*\<perm\>*)\] | Specify that the Git repository is to be shared amongst several users. This allows users belonging to the same group to push into that repository. When specified, the config variable **core.sharedRepository** is set so that files and directories under **\$GIT_DIR** are created with the requested permissions. When not specified, Git will use permissions reported by **umask**(2). | ❌ |

## TEMPLATE DIRECTORY

Files and directories in the template directory whose name do not start
with a dot will be copied to the **\$GIT_DIR** after it is created.

The template directory will be one of the following (in order):

> ·
>
> the argument given with the **--template** option;

> ·
>
> the contents of the **\$GIT_TEMPLATE_DIR** environment variable;

> ·
>
> the **init.templateDir** configuration variable; or

> ·
>
> the default template directory: **/usr/share/git-core/templates**.

The default template directory includes some directory structure,
suggested "exclude patterns" (see **gitignore**(5)), and sample hook
files.

The sample hooks are all disabled by default. To enable one of the
sample hooks rename it by removing its **.sample** suffix.

See **githooks**(5) for more general info on hook execution.

## EXAMPLES

Start a new Git repository for an existing code base

> > $ cd /path/to/my/codebase
> >     $ git init      (1)
> >     $ git add .     (2)
> >     $ git commit    (3)
>
> |        |                                                               |
> |-------:|:--------------------------------------------------------------|
> | **1.** | Create a **/path/to/my/codebase/.git** directory.             |
> | **2.** | Add all existing files to the index.                          |
> | **3.** | Record the pristine state as the first commit in the history. |

## CONFIGURATION

Everything below this line in this section is selectively included from
the **git-config**(1) documentation. The content is the same as what’s
found there:

**init.templateDir**

> Specify the directory from which templates will be copied.

**init.defaultBranch**

> Allows overriding the default branch name e.g. when initializing a new
> repository.

**init.defaultObjectFormat**

> Allows overriding the default object format for new repositories. See
> **--object-format=** in **git-init**(1). Both the command line option
> and the **GIT_DEFAULT_HASH** environment variable take precedence over
> this config.

**init.defaultRefFormat**

> Allows overriding the default ref storage format for new repositories.
> See **--ref-format=** in **git-init**(1). Both the command line option
> and the **GIT_DEFAULT_REF_FORMAT** environment variable take
> precedence over this config.

## GIT

Part of the **git**(1) suite

## Copyright

This documentation is derived from the Git man page for `git-init`.

Copyright (c) Git contributors. Licensed under the GNU General Public License version 2; see [Git's COPYING file](https://github.com/git/git/blob/master/COPYING).
