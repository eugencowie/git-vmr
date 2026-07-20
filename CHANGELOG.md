# Changelog

## 0.12.0

### 🚀 Features

- Add 'diff' command ([#106](https://github.com/eugencowie/git-vmr/pull/106))

### ⚙️ Miscellaneous Tasks

- Update skills ([#104](https://github.com/eugencowie/git-vmr/pull/104))

## 0.11.0

### 🚀 Features

- Make `push` only push branches with novel commits ([#102](https://github.com/eugencowie/git-vmr/pull/102))

### 🐛 Bug Fixes

- Make head resolution observe unborn branches ([#97](https://github.com/eugencowie/git-vmr/pull/97))

### 🚜 Refactor

- Unify git calls into a single git runner ([#73](https://github.com/eugencowie/git-vmr/pull/73))
- Add workspace module ([#74](https://github.com/eugencowie/git-vmr/pull/74))
- Unify path routing ([#75](https://github.com/eugencowie/git-vmr/pull/75))
- Add rendering module for command output ([#76](https://github.com/eugencowie/git-vmr/pull/76))
- Concentrate TOML persistence ([#77](https://github.com/eugencowie/git-vmr/pull/77))
- Derive event metadata from CLI definitions ([#78](https://github.com/eugencowie/git-vmr/pull/78))
- Make CLI context construction hermetic ([#80](https://github.com/eugencowie/git-vmr/pull/80))
- Extract command outcome reporting into module ([#81](https://github.com/eugencowie/git-vmr/pull/81))
- Unify the repo-list suffix rendering ([#82](https://github.com/eugencowie/git-vmr/pull/82))
- Eliminate the analytics builder ([#83](https://github.com/eugencowie/git-vmr/pull/83))
- Deepen the report policy ([#86](https://github.com/eugencowie/git-vmr/pull/86))
- Deepen the worktree lifecycle ([#85](https://github.com/eugencowie/git-vmr/pull/85))
- Deepen the update and analytics verticals ([#87](https://github.com/eugencowie/git-vmr/pull/87))
- Consolidate git head resolution ([#88](https://github.com/eugencowie/git-vmr/pull/88))
- Deepen the move domain ([#89](https://github.com/eugencowie/git-vmr/pull/89))
- Give analytics recording a sink seam ([#92](https://github.com/eugencowie/git-vmr/pull/92))
- Unify the command interface ([#93](https://github.com/eugencowie/git-vmr/pull/93))
- Restructure commands as vertical slices over shared cores ([#94](https://github.com/eugencowie/git-vmr/pull/94))
- Concentrate the aggregate policy at the routing seam ([#95](https://github.com/eugencowie/git-vmr/pull/95))
- Let result aggregation own the scope denominator ([#96](https://github.com/eugencowie/git-vmr/pull/96))
- Let root outcomes render their own dissolve fate ([#98](https://github.com/eugencowie/git-vmr/pull/98))
- Split foreach's child environment from the spawn ([#99](https://github.com/eugencowie/git-vmr/pull/99))
- Extract the run pipeline behind its own seam ([#100](https://github.com/eugencowie/git-vmr/pull/100))
- Extract a router view behind the workspace ([#101](https://github.com/eugencowie/git-vmr/pull/101))

### 🧪 Testing

- Add unit tests for mv/worktree ([#79](https://github.com/eugencowie/git-vmr/pull/79))
- Extract shared test fixtures ([#84](https://github.com/eugencowie/git-vmr/pull/84))
- Stabilise tag deletion tests ([#91](https://github.com/eugencowie/git-vmr/pull/91))

### ⚙️ Miscellaneous Tasks

- Update agent instructions ([#72](https://github.com/eugencowie/git-vmr/pull/72))
- Use standard location for tickets and decisions ([#90](https://github.com/eugencowie/git-vmr/pull/90))
- Update tickets location in repo ([#103](https://github.com/eugencowie/git-vmr/pull/103))

## 0.10.1

### 🐛 Bug Fixes

- Ensure analytics state is persisted to disk ([#64](https://github.com/eugencowie/git-vmr/pull/64))
- Fix update message formatting ([#66](https://github.com/eugencowie/git-vmr/pull/66))
- Handle existing inferred worktree branches ([#67](https://github.com/eugencowie/git-vmr/pull/67))

### 🎨 Styling

- Normalise behind-count switch output ([#68](https://github.com/eugencowie/git-vmr/pull/68))

## 0.10.0

### 🚀 Features

- Add anonymous, privacy-respecting usage analytics ([#61](https://github.com/eugencowie/git-vmr/pull/61))

### ⚙️ Miscellaneous Tasks

- Simplify workspace configuration ([#62](https://github.com/eugencowie/git-vmr/pull/62))

## 0.9.1

### 🐛 Bug Fixes

- Fix inconsistent tag list styling ([#59](https://github.com/eugencowie/git-vmr/pull/59))

## 0.9.0

### 🚀 Features

- Add global configuration file ([#56](https://github.com/eugencowie/git-vmr/pull/56))
- Check for updates automatically ([#50](https://github.com/eugencowie/git-vmr/pull/50))

## 0.8.0

### 🚀 Features

- Add missing flags to `add`, `mv`, and `rm` ([#45](https://github.com/eugencowie/git-vmr/pull/45))
- Add branch deletion flags to `worktree remove` ([#48](https://github.com/eugencowie/git-vmr/pull/48))
- Make `worktree` command with no arguments list worktrees ([#49](https://github.com/eugencowie/git-vmr/pull/49))

### 📚 Documentation

- Add copyright footers to generated documentation ([#43](https://github.com/eugencowie/git-vmr/pull/43))

### 🎨 Styling

- Standardise command output formatting ([#46](https://github.com/eugencowie/git-vmr/pull/46))
- Standardise help text formatting ([#47](https://github.com/eugencowie/git-vmr/pull/47))

## 0.7.0

### 🚀 Features

- Add `worktree` commands to add, move, remove and list worktrees ([#40](https://github.com/eugencowie/git-vmr/pull/40))
- Add `foreach` command to run arbitrary shell commands ([#42](https://github.com/eugencowie/git-vmr/pull/42))

## 0.6.0

### 🚀 Features

- Add `clone` command for cloning repositories ([#31](https://github.com/eugencowie/git-vmr/pull/31))
- Add `tag` command for creating, deleting, and listing tags ([#33](https://github.com/eugencowie/git-vmr/pull/33))
- Add `switch` command to switch between branches ([#34](https://github.com/eugencowie/git-vmr/pull/34))
- Add `fetch`, `pull`, and `push` commands for collaboration ([#35](https://github.com/eugencowie/git-vmr/pull/35))
- Add `reset` command to reset the index ([#36](https://github.com/eugencowie/git-vmr/pull/36))
- Add `--create` argument to `switch` command to create branches ([#37](https://github.com/eugencowie/git-vmr/pull/37))

### 🚜 Refactor

- Extract command handling into dedicated module ([#39](https://github.com/eugencowie/git-vmr/pull/39))

### 🎨 Styling

- Standardise output handling ([#38](https://github.com/eugencowie/git-vmr/pull/38))

## 0.5.0

### 🚀 Features

- Add branch deletion arguments ([#29](https://github.com/eugencowie/git-vmr/pull/29))

### 🐛 Bug Fixes

- Show correct binary name in status hints ([#25](https://github.com/eugencowie/git-vmr/pull/25))
- Fix inconsistent path separators ([#27](https://github.com/eugencowie/git-vmr/pull/27))

### 📚 Documentation

- Document supported commands ([#30](https://github.com/eugencowie/git-vmr/pull/30))

### 🎨 Styling

- Refine branch and status output styling ([#28](https://github.com/eugencowie/git-vmr/pull/28))

## 0.4.0

### 🚀 Features

- Add destination argument to `init` command ([#14](https://github.com/eugencowie/git-vmr/pull/14))
- Add `branch` command to create and list branches ([#16](https://github.com/eugencowie/git-vmr/pull/16))
- Add `commit` command to create commits for staged changes ([#17](https://github.com/eugencowie/git-vmr/pull/17))
- Add `merge` command to merge changes ([#18](https://github.com/eugencowie/git-vmr/pull/18))
- Add `rebase` command to apply commits on top of another tip ([#19](https://github.com/eugencowie/git-vmr/pull/19))

### 🚜 Refactor

- Centralise git command execution ([#21](https://github.com/eugencowie/git-vmr/pull/21))
- Centralise repository discovery ([#22](https://github.com/eugencowie/git-vmr/pull/22))
- Standardise git command result handling ([#23](https://github.com/eugencowie/git-vmr/pull/23))
- Restructure git command organisation ([#24](https://github.com/eugencowie/git-vmr/pull/24))

### ⚡ Performance

- Run commands in parallel ([#20](https://github.com/eugencowie/git-vmr/pull/20))

## 0.3.0

### 🚀 Features

- Add `add`, `mv`, `restore`, `rm` commands for managing the working tree ([#11](https://github.com/eugencowie/git-vmr/pull/11))

### ⚙️ Miscellaneous Tasks

- Rewrite homepage install commands ([#13](https://github.com/eugencowie/git-vmr/pull/13))

## 0.2.0

### 🚀 Features

- Add working directory argument ([#9](https://github.com/eugencowie/git-vmr/pull/9))
- Add `status` command to show the virtual monorepo status ([#10](https://github.com/eugencowie/git-vmr/pull/10))

### 📚 Documentation

- Update changelog format ([#7](https://github.com/eugencowie/git-vmr/pull/7))

## 0.1.0

### 🚀 Features

- Add `init` command for creating virtual monorepo config ([#5](https://github.com/eugencowie/git-vmr/pull/5))

### ⚙️ Miscellaneous Tasks

- Create project structure ([#1](https://github.com/eugencowie/git-vmr/pull/1))
- Configure dependency updates ([#2](https://github.com/eugencowie/git-vmr/pull/2))
- Configure release process ([#3](https://github.com/eugencowie/git-vmr/pull/3))
