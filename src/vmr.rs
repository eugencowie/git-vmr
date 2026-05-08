use crate::config::Config;
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

mod repo;

pub use repo::Repo;

#[derive(Debug)]
pub struct Vmr
{
    pub path: PathBuf,
    config: OnceLock<Config>
}

impl Vmr
{
    pub fn find(working_dir: &Path) -> Result<Vmr>
    {
        // Search working directory and ancestors
        for parent in working_dir.ancestors()
        {
            if parent.join(".gitvmr").exists()
            {
                return Ok(Vmr {
                    path: parent.to_path_buf(),
                    config: OnceLock::new()
                });
            }
        }

        // Report missing VMR root
        bail!(
            "not a virtual monorepo (or any of the parent directories): .gitvmr"
        );
    }

    #[allow(dead_code)]
    pub fn config(&self) -> Result<&Config>
    {
        if let Some(config) = self.config.get()
        {
            return Ok(config);
        }

        let config_path = self.path.join(".gitvmr/config");
        let contents = fs::read_to_string(&config_path).with_context(|| {
            format!("failed to read config '{}'", config_path.display())
        })?;
        let config = toml::from_str(&contents).with_context(|| {
            format!("failed to parse config '{}'", config_path.display())
        })?;
        let _ = self.config.set(config);

        self.config.get().context("failed to cache VMR config after loading")
    }

    pub fn repos(&self) -> Result<Vec<Repo>>
    {
        find_vmr_repos(&self.path)
    }

    pub fn route_paths(
        &self,
        working_dir: &Path,
        paths: &[PathBuf]
    ) -> Result<BTreeMap<Repo, Vec<PathBuf>>>
    {
        route_paths(working_dir, &self.path, paths)
    }

    pub fn route_single_path(
        &self,
        working_dir: &Path,
        path: &Path
    ) -> Result<(Repo, PathBuf)>
    {
        route_single_path(working_dir, &self.path, path)
    }
}

pub fn find_vmr_repos(vmr_root: &Path) -> Result<Vec<Repo>>
{
    let mut repos = Vec::new();

    for repo_path in child_dirs(vmr_root)?
    {
        if let Some(repo) = Repo::from_child_dir(repo_path)?
        {
            repos.push(repo);
        }
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(repos)
}

pub fn child_dirs(vmr_root: &Path) -> Result<Vec<PathBuf>>
{
    Ok(fs::read_dir(vmr_root)
        .with_context(|| {
            format!("failed to read VMR root '{}'", vmr_root.display())
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|entry| entry.file_name() != ".gitvmr")
        .map(|entry| entry.path())
        .collect::<Vec<_>>())
}

pub fn route_paths(
    working_dir: &Path,
    vmr_root: &Path,
    paths: &[PathBuf]
) -> Result<BTreeMap<Repo, Vec<PathBuf>>>
{
    let mut grouped: BTreeMap<Repo, Vec<PathBuf>> = BTreeMap::new();

    // Resolve and route every path before mutating any repository
    for path in paths
    {
        let normalized = resolve_path(working_dir, path).clean();
        let routed = route_path(vmr_root, &normalized)
            .with_context(|| format!("failed to route '{}'", path.display()))?;

        for (repo_path, repo_relative_path) in routed
        {
            grouped.entry(repo_path).or_default().push(repo_relative_path);
        }
    }

    Ok(grouped)
}

pub fn route_single_path(
    working_dir: &Path,
    vmr_root: &Path,
    path: &Path
) -> Result<(Repo, PathBuf)>
{
    // Resolve one operand without allowing aggregate VMR root expansion
    let normalized = resolve_path(working_dir, path).clean();

    if normalized == vmr_root
    {
        bail!("'{}' resolves to the VMR root aggregate", path.display());
    }

    // Reuse normal child repository ownership checks
    route_path(vmr_root, &normalized)
        .with_context(|| format!("failed to route '{}'", path.display()))?
        .into_iter()
        .next()
        .context("path did not route to a child repository")
}

pub fn resolve_path(working_dir: &Path, path: &Path) -> PathBuf
{
    // Interpret relative paths against effective working directory
    if path.is_absolute() { path.to_path_buf() } else { working_dir.join(path) }
}

pub fn route_path(vmr_root: &Path, path: &Path)
-> Result<Vec<(Repo, PathBuf)>>
{
    // Expand the aggregate VMR root view across child repositories
    if path == vmr_root
    {
        return expand_vmr_root(vmr_root);
    }

    // Ensure path stays inside the VMR root
    let relative = path.strip_prefix(vmr_root).with_context(|| {
        format!(
            "'{}' is outside virtual monorepo '{}'",
            path.display(),
            vmr_root.display()
        )
    })?;

    // Use first path component as owning child repository
    let mut components = relative.components();
    let repo_name = match components.next()
    {
        Some(Component::Normal(name)) => name,
        _ => bail!("'{}' is not owned by a child repository", path.display())
    };

    // Reject VMR metadata paths
    if repo_name == OsStr::new(".gitvmr")
    {
        bail!(
            "cannot route virtual monorepo metadata path '{}'",
            path.display()
        );
    }

    // Require explicit paths to be owned by child Git repositories
    let repo_path = vmr_root.join(repo_name);
    let repo = Repo::from_child_dir(repo_path)?.with_context(|| {
        format!("'{}' is not owned by a child Git repository", path.display())
    })?;
    // Build path relative to owning repository
    let mut repo_relative_path = PathBuf::new();
    for component in components
    {
        repo_relative_path.push(component.as_os_str());
    }
    if repo_relative_path.as_os_str().is_empty()
    {
        repo_relative_path.push(".");
    }

    Ok(vec![(repo, repo_relative_path)])
}

fn expand_vmr_root(vmr_root: &Path) -> Result<Vec<(Repo, PathBuf)>>
{
    let mut repos = Vec::new();

    for repo in find_vmr_repos(vmr_root)?
    {
        repos.push((repo, PathBuf::from(".")));
    }

    // Keep repository order deterministic
    repos.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));
    Ok(repos)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;

    #[test]
    fn finds_marker_in_start_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();

        // Act
        let result = Vmr::find(tmp.path()).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn finds_marker_in_ancestor()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("repo/src");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();

        // Act
        let result = Vmr::find(&nested).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn errors_when_marker_is_missing()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let err = Vmr::find(tmp.path()).unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("not a virtual monorepo"));
    }

    #[test]
    fn config_parses_on_demand()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::write(tmp.path().join(".gitvmr/config"), "[core]\nversion = 7\n")
            .unwrap();
        let vmr = Vmr::find(tmp.path()).unwrap();

        // Act
        let config = vmr.config().unwrap();

        // Assert
        assert_eq!(config.core.version, 7);
    }

    #[test]
    fn missing_config_errors_only_when_config_is_called()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        let vmr = Vmr::find(tmp.path()).unwrap();

        // Act
        let err = vmr.config().unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("failed to read config"));
    }

    #[test]
    fn malformed_config_errors_only_when_config_is_called()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::write(tmp.path().join(".gitvmr/config"), "not toml = [").unwrap();
        let vmr = Vmr::find(tmp.path()).unwrap();

        // Act
        let err = vmr.config().unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("failed to parse config"));
    }

    fn vmr_fixture() -> tempfile::TempDir
    {
        // Arrange fixture with Git and non-Git children
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("backend")).unwrap();
        fs::create_dir(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir(tmp.path().join("frontend")).unwrap();
        fs::create_dir(tmp.path().join("frontend/.git")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        fs::write(tmp.path().join("README.md"), "vmr\n").unwrap();
        tmp
    }

    #[test]
    fn normalizes_paths_lexically_without_requiring_existence()
    {
        // Act
        let path = Path::new("/tmp/vmr/backend/../backend/deleted.rs").clean();

        // Assert
        assert_eq!(path, PathBuf::from("/tmp/vmr/backend/deleted.rs"));
    }

    #[test]
    fn resolves_relative_paths_against_working_dir()
    {
        // Act
        let path = resolve_path(
            Path::new("/tmp/vmr/frontend"),
            Path::new("../backend/a.rs")
        );

        // Assert
        assert_eq!(path, PathBuf::from("/tmp/vmr/frontend/../backend/a.rs"));
    }

    #[test]
    fn routes_repo_root_to_dot()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let routed =
            route_path(tmp.path(), &tmp.path().join("backend")).unwrap();

        // Assert
        assert_eq!(routed, vec![(
            repo(tmp.path(), "backend"),
            PathBuf::from(".")
        )]);
    }

    #[test]
    fn routes_child_path_to_repo_relative_path()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let routed =
            route_path(tmp.path(), &tmp.path().join("backend/src/main.rs"))
                .unwrap();

        // Assert
        assert_eq!(routed, vec![(
            repo(tmp.path(), "backend"),
            PathBuf::from("src/main.rs")
        )]);
    }

    #[test]
    fn expands_vmr_root_to_immediate_child_git_repos()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let routed = route_path(tmp.path(), tmp.path()).unwrap();

        // Assert
        assert_eq!(routed, vec![
            (repo(tmp.path(), "backend"), PathBuf::from(".")),
            (repo(tmp.path(), "frontend"), PathBuf::from("."))
        ]);
    }

    #[test]
    fn vmr_repos_skips_non_git_dirs_and_sorts_by_name()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("zeta")).unwrap();
        fs::create_dir(tmp.path().join("zeta/.git")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        fs::create_dir(tmp.path().join("alpha")).unwrap();
        fs::create_dir(tmp.path().join("alpha/.git")).unwrap();
        let vmr = Vmr::find(tmp.path()).unwrap();

        // Act
        let repos = vmr.repos().unwrap();

        // Assert
        assert_eq!(repos, vec![
            repo(tmp.path(), "alpha"),
            repo(tmp.path(), "zeta")
        ]);
    }

    #[test]
    fn rejects_paths_outside_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let err =
            route_path(tmp.path(), &tmp.path().join("../outside.txt").clean())
                .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("outside virtual monorepo"));
    }

    #[test]
    fn rejects_vmr_metadata_paths()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let err = route_path(tmp.path(), &tmp.path().join(".gitvmr/config"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("metadata"));
    }

    #[test]
    fn rejects_non_git_child_paths()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let err = route_path(tmp.path(), &tmp.path().join("docs/readme.md"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("child Git repository"));
    }

    #[test]
    fn rejects_root_files()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let err = route_path(tmp.path(), &tmp.path().join("README.md"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("child Git repository"));
    }

    #[test]
    fn groups_routes_by_repository()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let routed = route_paths(tmp.path(), tmp.path(), &[
            PathBuf::from("backend/src/main.rs"),
            PathBuf::from("backend/lib.rs"),
            PathBuf::from("frontend/app.rs")
        ])
        .unwrap();

        // Assert
        assert_eq!(routed.get(&repo(tmp.path(), "backend")).unwrap(), &vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("lib.rs")
        ]);
        assert_eq!(routed.get(&repo(tmp.path(), "frontend")).unwrap(), &vec![
            PathBuf::from("app.rs")
        ]);
    }

    #[test]
    fn routes_single_path_without_vmr_root_expansion()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let routed = route_single_path(
            tmp.path(),
            tmp.path(),
            Path::new("backend/src/main.rs")
        )
        .expect("path should route");

        // Assert
        assert_eq!(
            routed,
            (repo(tmp.path(), "backend"), PathBuf::from("src/main.rs"))
        );
    }

    #[test]
    fn route_single_path_rejects_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();

        // Act
        let err = route_single_path(tmp.path(), tmp.path(), Path::new("."))
            .expect_err("VMR root should fail");

        // Assert
        assert!(format!("{err:#}").contains("VMR root aggregate"));
    }

    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }
}
