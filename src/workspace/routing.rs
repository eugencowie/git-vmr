use crate::vmr::Repo;
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// The declared per-command rule for what happens when a user-supplied
/// path resolves to the aggregate path: allowed to expand across the
/// workspace, or denied with a command-supplied message.
pub enum AggregatePolicy
{
    Allow,
    Deny(String)
}

/// What a path-taking command operates over: explicit paths with a
/// declared aggregate policy, or the entire VMR chosen deliberately via
/// the aggregate path.
pub enum Scope
{
    Paths
    {
        paths: Vec<PathBuf>,
        aggregate: AggregatePolicy
    },
    EntireVmr
}

/// The router: a borrowed view over a workspace's fixed snapshot — the
/// VMR root, the child repos, and the working dir the workspace was
/// opened from — turning a scope into owning child repos with
/// repo-relative paths. Private to the workspace core; slices reach
/// routing only through the workspace.
pub struct Router<'a>
{
    root: &'a Path,
    repos: &'a [Repo],
    working_dir: &'a Path
}

impl<'a> Router<'a>
{
    pub fn new(
        root: &'a Path,
        repos: &'a [Repo],
        working_dir: &'a Path
    ) -> Router<'a>
    {
        Router { root, repos, working_dir }
    }

    /// Resolves a user-supplied path against the working dir and
    /// normalizes it lexically. The only form in which user paths reach
    /// the filesystem or routing.
    pub fn target(&self, path: &Path) -> PathBuf
    {
        let resolved = if path.is_absolute()
        {
            path.to_path_buf()
        }
        else
        {
            self.working_dir.join(path)
        };
        resolved.clean()
    }

    /// Routes the scope to its owning child repos, grouping repo-relative
    /// paths per repo. Paths route one by one so the first problem in
    /// path order wins, enforcing the aggregate policy as each path is
    /// reached.
    pub fn route(&self, scope: Scope) -> Result<BTreeMap<Repo, Vec<PathBuf>>>
    {
        let (paths, aggregate) = match scope
        {
            Scope::Paths { paths, aggregate } => (paths, aggregate),
            // The aggregate path expands to every snapshotted child repo
            Scope::EntireVmr =>
            {
                return Ok(self
                    .repos
                    .iter()
                    .map(|repo| (repo.clone(), vec![PathBuf::from(".")]))
                    .collect());
            }
        };

        let mut grouped: BTreeMap<Repo, Vec<PathBuf>> = BTreeMap::new();

        for path in paths
        {
            let target = self.target(&path);

            if let AggregatePolicy::Deny(message) = &aggregate
                && self.is_aggregate(&target)
            {
                bail!("{message}");
            }

            let routed = self.route_target(&target).with_context(|| {
                format!("error: failed to route '{}'", path.display())
            })?;

            for (repo, repo_path) in routed
            {
                grouped.entry(repo).or_default().push(repo_path);
            }
        }

        Ok(grouped)
    }

    /// Routes a single operand to its owning child repo. Single-operand
    /// routing always denies the aggregate path: one operand cannot
    /// expand to many child repos.
    pub fn route_single(&self, path: &Path) -> Result<(Repo, PathBuf)>
    {
        let target = self.target(path);

        if self.is_aggregate(&target)
        {
            bail!(
                "error: '{}' resolves to the VMR root aggregate",
                path.display()
            );
        }

        // Reuse normal child repository ownership checks
        self.route_target(&target)
            .with_context(|| {
                format!("error: failed to route '{}'", path.display())
            })?
            .into_iter()
            .next()
            .context("error: path did not route to a child repository")
    }

    /// Whether a target is the aggregate path: the VMR root itself.
    fn is_aggregate(&self, target: &Path) -> bool
    {
        target == self.root
    }

    fn route_target(&self, target: &Path) -> Result<Vec<(Repo, PathBuf)>>
    {
        // Expand the aggregate VMR root view across child repositories
        if self.is_aggregate(target)
        {
            return Ok(self
                .repos
                .iter()
                .map(|repo| (repo.clone(), PathBuf::from(".")))
                .collect());
        }

        // Ensure path stays inside the VMR root
        let relative = target.strip_prefix(self.root).with_context(|| {
            format!(
                "error: '{}' is outside virtual monorepo '{}'",
                target.display(),
                self.root.display()
            )
        })?;

        // Use first path component as owning child repository
        let mut components = relative.components();
        let repo_name = match components.next()
        {
            Some(Component::Normal(name)) => name,
            _ =>
            {
                bail!(
                    "error: '{}' is not owned by a child repository",
                    target.display()
                );
            }
        };

        // Reject VMR metadata paths
        if repo_name == ".gitvmr"
        {
            bail!(
                "error: cannot route virtual monorepo metadata path '{}'",
                target.display()
            );
        }

        // Require explicit paths to be owned by snapshotted child repositories
        let repo_path = self.root.join(repo_name);
        let repo = self
            .repos
            .iter()
            .find(|repo| repo.path == repo_path)
            .cloned()
            .with_context(|| {
                format!(
                    "error: '{}' is not owned by a child Git repository",
                    target.display()
                )
            })?;

        // Build path relative to owning repository
        let mut repo_relative_path = components.as_path().to_path_buf();
        if repo_relative_path.as_os_str().is_empty()
        {
            repo_relative_path.push(".");
        }

        Ok(vec![(repo, repo_relative_path)])
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    // Routing is pure path math: fixtures are literal paths, no
    // filesystem or git involved.
    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }

    fn repos(root: &Path) -> Vec<Repo>
    {
        vec![repo(root, "backend"), repo(root, "frontend")]
    }

    #[test]
    fn routes_repo_root_to_dot()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed = router.route_single(Path::new("backend")).unwrap();

        // Assert
        assert_eq!(routed, (repo(root, "backend"), PathBuf::from(".")));
    }

    #[test]
    fn routes_child_path_to_repo_relative_path()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed =
            router.route_single(Path::new("backend/src/main.rs")).unwrap();

        // Assert
        assert_eq!(
            routed,
            (repo(root, "backend"), PathBuf::from("src/main.rs"))
        );
    }

    #[test]
    fn groups_routes_by_repository()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed = router
            .route(Scope::Paths {
                paths: vec![
                    PathBuf::from("backend/src/main.rs"),
                    PathBuf::from("backend/lib.rs"),
                    PathBuf::from("frontend/app.rs"),
                ],
                aggregate: AggregatePolicy::Allow
            })
            .unwrap();

        // Assert
        assert_eq!(routed.get(&repo(root, "backend")).unwrap(), &vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("lib.rs")
        ]);
        assert_eq!(routed.get(&repo(root, "frontend")).unwrap(), &vec![
            PathBuf::from("app.rs")
        ]);
    }

    #[test]
    fn entire_vmr_scope_expands_to_every_child_repo()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed = router.route(Scope::EntireVmr).unwrap();

        // Assert
        assert_eq!(routed.into_iter().collect::<Vec<_>>(), vec![
            (repo(root, "backend"), vec![PathBuf::from(".")]),
            (repo(root, "frontend"), vec![PathBuf::from(".")])
        ]);
    }

    #[test]
    fn allow_policy_expands_the_aggregate_path_to_every_child_repo()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed = router
            .route(Scope::Paths {
                paths: vec![PathBuf::from(".")],
                aggregate: AggregatePolicy::Allow
            })
            .unwrap();

        // Assert
        assert_eq!(routed.len(), 2);
    }

    #[test]
    fn deny_policy_rejects_the_aggregate_path()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let backend = root.join("backend");

        // The aggregate path is denied however the user spells it
        for (working_dir, path) in [(root, "."), (backend.as_path(), "..")]
        {
            // Arrange
            let router = Router::new(root, &repos, working_dir);

            // Act
            let err = router
                .route(Scope::Paths {
                    paths: vec![PathBuf::from(path)],
                    aggregate: AggregatePolicy::Deny("denied".to_owned())
                })
                .unwrap_err();

            // Assert
            assert_eq!(err.to_string(), "denied");
        }
    }

    #[test]
    fn deny_policy_ignores_paths_owned_by_a_child_repo()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let routed = router
            .route(Scope::Paths {
                paths: vec![PathBuf::from("backend/src/main.rs")],
                aggregate: AggregatePolicy::Deny("denied".to_owned())
            })
            .unwrap();

        // Assert
        assert_eq!(routed.len(), 1);
        assert!(routed.contains_key(&repo(root, "backend")));
    }

    #[test]
    fn routing_problems_surface_in_path_order()
    {
        // Arrange: the unroutable path precedes the denied aggregate path
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let err = router
            .route(Scope::Paths {
                paths: vec![PathBuf::from("nonexistent"), PathBuf::from(".")],
                aggregate: AggregatePolicy::Deny("denied".to_owned())
            })
            .unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("failed to route 'nonexistent'"));
    }

    #[test]
    fn rejects_paths_outside_vmr_root()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let err = router
            .route_single(Path::new("../outside.txt"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("outside virtual monorepo"));
    }

    #[test]
    fn rejects_vmr_metadata_paths()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let err = router
            .route_single(Path::new(".gitvmr/config"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("metadata"));
    }

    #[test]
    fn rejects_paths_not_owned_by_a_child_repo()
    {
        // Arrange: docs is not a snapshotted child repo, README.md is a
        // root file
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        for path in ["docs/readme.md", "README.md"]
        {
            // Act
            let err = router
                .route_single(Path::new(path))
                .expect_err("path should fail");

            // Assert
            assert!(format!("{err:#}").contains("child Git repository"));
        }
    }

    #[test]
    fn route_single_rejects_the_aggregate_path()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let router = Router::new(root, &repos, root);

        // Act
        let err = router.route_single(Path::new(".")).unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("VMR root aggregate"));
    }

    #[test]
    fn target_resolves_relative_paths_against_working_dir_and_normalizes()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let working_dir = root.join("frontend");
        let router = Router::new(root, &repos, &working_dir);

        // Act
        let target = router.target(Path::new("../backend/a.rs"));

        // Assert
        assert_eq!(target, PathBuf::from("/vmr/backend/a.rs"));
    }

    #[test]
    fn target_keeps_absolute_paths()
    {
        // Arrange
        let root = Path::new("/vmr");
        let repos = repos(root);
        let working_dir = root.join("frontend");
        let router = Router::new(root, &repos, &working_dir);

        // Act
        let target = router.target(Path::new("/vmr/backend/../backend/a.rs"));

        // Assert
        assert_eq!(target, PathBuf::from("/vmr/backend/a.rs"));
    }
}
