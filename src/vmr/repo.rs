use super::Declaration;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// A member repo: where it lives on disk, its path from the VMR root, which
/// output is keyed by (`.` for the root repo, the manifest path for a child
/// repo), and which kind of member it is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repo
{
    pub absolute_path: PathBuf,
    pub path: String,
    pub kind: RepoKind
}

/// Which kind of member repo a repo is. Only diff, foreach, and
/// worktrees branch on it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RepoKind
{
    Root,
    Child
    {
        name: String
    }
}

impl Repo
{
    /// The root repo, whose working tree is the VMR root `root`.
    pub fn root(root: &Path) -> Repo
    {
        Repo {
            absolute_path: root.to_path_buf(),
            path: ".".to_owned(),
            kind: RepoKind::Root
        }
    }

    /// The child repo `declaration` declares at the VMR root `root`.
    pub fn child(root: &Path, declaration: &Declaration) -> Repo
    {
        Repo {
            absolute_path: root.join(declaration.path.as_str()),
            path: declaration.path.to_string(),
            kind: RepoKind::Child { name: declaration.name.clone() }
        }
    }
}

/// The root repo first, then child repos by path. Sorting by path alone
/// would not do: `.` does not reliably sort first, as `-` and `!` precede
/// it.
impl Ord for Repo
{
    fn cmp(&self, other: &Repo) -> Ordering
    {
        // Put the root repo first, then sort by absolute path
        let is_child = |repo: &Repo| repo.kind != RepoKind::Root;
        is_child(self)
            .cmp(&is_child(other))
            .then_with(|| self.absolute_path.cmp(&other.absolute_path))
            // Break remaining ties on every field, so ordering agrees with
            // equality
            .then_with(|| self.path.cmp(&other.path))
            .then_with(|| self.kind.cmp(&other.kind))
    }
}

impl PartialOrd for Repo
{
    fn partial_cmp(&self, other: &Repo) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::manifest;

    #[test]
    fn sorts_the_root_repo_first_then_children_by_path()
    {
        // Arrange: `-` and `!` sort before `.`
        let root = Path::new("/vmr");
        let manifest = manifest(&["frontend", "-dash", "backend", "!bang"]);
        let mut repos = manifest
            .declarations()
            .iter()
            .map(|declaration| Repo::child(root, declaration))
            .rev()
            .chain([Repo::root(root)])
            .collect::<Vec<_>>();

        // Act
        repos.sort();

        // Assert
        assert_eq!(
            repos.iter().map(|repo| repo.path.as_str()).collect::<Vec<_>>(),
            [".", "!bang", "-dash", "backend", "frontend"]
        );
    }
}
