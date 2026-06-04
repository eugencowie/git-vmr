# Changelog

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
