# git-vmr: multi-repo flexibility with monorepo ergonomics

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
| [clone](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-clone.md) | Clone a repository into a new directory | ✅ |
| [init](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-init.md) | Create an empty virtual monorepo or reinitialize an existing one | ✅ |

### Work on the current change

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [add](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-add.md) | Add file contents to the index | ✅ |
| [mv](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-mv.md) | Move or rename a file, a directory, or a symlink | ✅ |
| [restore](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-restore.md) | Restore working tree files | ✅ |
| [rm](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-rm.md) | Remove files from the working tree and from the index | ✅ |

### Examine the history and state

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| bisect  | Use binary search to find the commit that introduced a bug | ❌ |
| diff    | Show changes between commits, commit and working tree, etc | ❌ |
| grep    | Print lines matching a pattern | ❌ |
| log     | Show commit logs | ❌ |
| show    | Show various types of objects | ❌ |
| [status](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-status.md) | Show the working tree status | ✅ |

### Grow, mark and tweak your common history

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| backfill| Download missing objects in a partial clone | ❌ |
| [branch](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-branch.md) | List, create, or delete branches | ✅ |
| [commit](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-commit.md) | Record changes to the repository | ✅ |
| [merge](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-merge.md) | Join two or more development histories together | ✅ |
| [rebase](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-rebase.md) | Reapply commits on top of another base tip | ✅ |
| [reset](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-reset.md) | Set `HEAD` or the index to a known state | ✅ |
| [switch](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-switch.md) | Switch branches | ✅ |
| [tag](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-tag.md) | Create, list, delete or verify tags | ✅ |

### Collaborate

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [fetch](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-fetch.md) | Download objects and refs from another repository | ✅ |
| [pull](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-pull.md) | Fetch from and integrate with another repository or a local branch | ✅ |
| [push](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-push.md) | Update remote refs along with associated objects | ✅ |

### Unique commands

| Command | Description | Supported? |
| ------- | ----------- | ---------- |
| [foreach](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git-submodule.md) | Evaluates an arbitrary shell command in each checked out repository | ✅ |

[docs/parity/git.md](https://github.com/eugencowie/git-vmr/blob/main/docs/parity/git.md) lists available subcommands. See `docs/parity/git-<command>.md` to read about a specific subcommand or concept.

## Analytics

To support early development, versions prior to `1.0` have anonymous, privacy-respecting usage analytics powered by [Aptabase](https://aptabase.com) enabled by default. Once version `1.0` is released, analytics will be disabled by default on both new and existing installations unless you have explicitly opted in.

Collected data consists of a single `command_finished` event with `name`, `success`, `duration_ms`, and boolean properties for used flags such as `force_flag` and `working_dir_global_flag`. No flag values, paths, repository names, remotes, branches, commit identifiers, commit messages, shell commands, or error text are collected. The Aptabase SDK annotates each event with the app version, SDK version, operating system name, operating system version, locale, debug flag, timestamp, and session ID. Session IDs are reused across command invocations within Aptabase's four-hour session window.

To opt out, add this to your `~/.config/git-vmr/config.toml`:

```toml
[analytics]
enabled = false
```

## License

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; version 2.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, see <https://www.gnu.org/licenses/>.
