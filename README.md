# git-vmr: multi-repo flexibility with monorepo ergonomics

> *This project follows the [ai-disclosure convention](https://github.com/ggfevans/ai-disclosure): see [AI_DISCLOSURE.md](https://github.com/eugencowie/git-vmr/blob/develop/AI_DISCLOSURE.md)*.

git-vmr is a Git CLI wrapper for interacting with multiple repositories as a single unified workspace, combining the flexibility of independent repositories with the ergonomics of a monorepo.

<div class="oranda-hide">

## Install

On macOS and Linux:

```sh
curl https://eugencowie.github.io/git-vmr/install.sh | sh
```

On Windows in PowerShell:

```powershell
irm https://eugencowie.github.io/git-vmr/install.ps1 | iex
```

</div>

## Commands

These are common git-vmr commands used in various situations:

### Start a working area

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [clone](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/clone.md) | Clone a repository into a new directory | ✅ |
| [init](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/init.md) | Create an empty virtual monorepo or reinitialize an existing one | ✅ |

### Work on the current change

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [add](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/add.md) | Add file contents to the index | ✅ |
| [mv](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/mv.md) | Move or rename a file, a directory, or a symlink | ✅ |
| [restore](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/restore.md) | Restore working tree files | ✅ |
| [rm](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/rm.md) | Remove files from the working tree and from the index | ✅ |

### Examine the history and state

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| bisect  | Use binary search to find the commit that introduced a bug | ❌ |
| diff    | Show changes between commits, commit and working tree, etc | ❌ |
| grep    | Print lines matching a pattern | ❌ |
| log     | Show commit logs | ❌ |
| show    | Show various types of objects | ❌ |
| [status](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/status.md) | Show the working tree status | ✅ |

### Grow, mark and tweak your common history

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| backfill| Download missing objects in a partial clone | ❌ |
| [branch](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/branch.md) | List, create, or delete branches | ✅ |
| [commit](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/commit.md) | Record changes to the repository | ✅ |
| [merge](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/merge.md) | Join two or more development histories together | ✅ |
| [rebase](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/rebase.md) | Reapply commits on top of another base tip | ✅ |
| reset   | Set `HEAD` or the index to a known state | ❌ |
| switch  | Switch branches | ❌ |
| [tag](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr/tag.md) | Create, list, delete or verify tags | ✅ |

### Collaborate

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| fetch   | Download objects and refs from another repository | ❌ |
| pull    | Fetch from and integrate with another repository or a local branch | ❌ |
| push    | Update remote refs along with associated objects | ❌ |

[docs/git-vmr.md](https://github.com/eugencowie/git-vmr/blob/develop/docs/git-vmr.md) lists available subcommands. See `docs/git-vmr/<command>.md` to read about a specific subcommand or concept.

## License

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; version 2.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, see <https://www.gnu.org/licenses/>.
