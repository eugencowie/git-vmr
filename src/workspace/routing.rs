use crate::vmr::{MANIFEST, Membership, Owner, Repo, RepoKind};
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
/// VMR root, its membership, and the working dir the workspace was opened
/// from — turning a scope into owning child repos with
/// repo-relative paths. Private to the workspace core; slices reach
/// routing only through the workspace.
pub struct Router<'a>
{
    root: &'a Path,
    membership: &'a Membership,
    working_dir: &'a Path
}

impl<'a> Router<'a>
{
    pub fn new(
        root: &'a Path,
        membership: &'a Membership,
        working_dir: &'a Path
    ) -> Router<'a>
    {
        Router { root, membership, working_dir }
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
                    .membership
                    .cloned()
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
                .membership
                .cloned()
                .iter()
                .map(|repo| (repo.clone(), PathBuf::from(".")))
                .collect());
        }

        // Route only into cloned child repositories
        match self.classify(target)?
        {
            Classification::Member(repo, path)
                if repo.kind == RepoKind::Root
                    && path.starts_with(MANIFEST) =>
                bail!(
                    "error: cannot route virtual monorepo metadata path '{}'",
                    target.display()
                ),
            Classification::Member(repo, path)
                if repo.kind != RepoKind::Root =>
                Ok(vec![(repo.clone(), path)]),
            Classification::ChildRoot(repo) =>
                Ok(vec![(repo.clone(), PathBuf::from("."))]),
            Classification::Member(..) | Classification::Undeclared { .. } =>
                bail!(
                    "error: '{}' is not owned by a child Git repository",
                    target.display()
                ),
        }
    }

    /// Classifies a target by what owns it: a member repo, or an undeclared
    /// repo, with the path relative to it, or a child root itself. Fails
    /// for a target outside the VMR or inside a missing child repo.
    pub fn classify(&self, target: &Path) -> Result<Classification<'a>>
    {
        // Ensure path stays inside the VMR root
        let relative = target.strip_prefix(self.root).with_context(|| {
            format!(
                "error: '{}' is outside virtual monorepo '{}'",
                target.display(),
                self.root.display()
            )
        })?;

        // Take the first path component as the first-level entry; the VMR
        // root itself belongs to the root repo
        let mut components = relative.components();
        let Some(Component::Normal(entry)) = components.next()
        else
        {
            return Ok(Classification::Member(
                self.membership.root(),
                PathBuf::from(".")
            ));
        };
        let rest = components.as_path();

        // Classify by the repo that owns the entry, if any
        Ok(match self.membership.owner(entry)
        {
            Some(Owner::Cloned(repo)) if rest.as_os_str().is_empty() =>
                Classification::ChildRoot(repo),
            Some(Owner::Cloned(repo)) =>
                Classification::Member(repo, rest.to_path_buf()),
            Some(Owner::Missing(declaration)) => bail!(
                "error: '{}' is not cloned; run 'git vmr clone' to clone it",
                declaration.path
            ),
            Some(Owner::Undeclared(name)) =>
                Classification::Undeclared { name, path: dot_if_empty(rest) },
            None => Classification::Member(
                self.membership.root(),
                relative.to_path_buf()
            )
        })
    }
}

/// What owns a target, as the router classifies it.
#[derive(Debug, PartialEq)]
pub enum Classification<'a>
{
    /// Inside a member repo — a cloned child repo, otherwise the root repo —
    /// with the path relative to it.
    Member(&'a Repo, PathBuf),
    /// A cloned child repo's root directory.
    ChildRoot(&'a Repo),
    /// Inside a first-level Git repository the manifest does not declare,
    /// with the path relative to it.
    Undeclared
    {
        name: &'a str, path: PathBuf
    }
}

/// `path`, or `.` when it is empty.
fn dot_if_empty(path: &Path) -> PathBuf
{
    if path.as_os_str().is_empty()
    {
        PathBuf::from(".")
    }
    else
    {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::manifest;

    // Routing is pure path math: fixtures are literal paths, no
    // filesystem or git involved.
    fn repo(root: &Path, name: &str) -> Repo
    {
        crate::test_support::repo(name, root.join(name))
    }

    fn membership(root: &Path) -> Membership
    {
        Membership::new(
            root,
            &manifest(&["api", "backend", "frontend"]),
            ["backend", "frontend", "scratch"].map(str::to_owned)
        )
    }

    #[test]
    fn classifies_a_path_inside_a_child_repo_relative_to_it()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        // Act
        let class = router.classify(Path::new("/vmr/backend/src/main.rs"));

        // Assert
        assert_eq!(
            class.unwrap(),
            Classification::Member(
                &repo(root, "backend"),
                PathBuf::from("src/main.rs")
            )
        );
    }

    #[test]
    fn classifies_a_child_root()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        // Act
        let class = router.classify(Path::new("/vmr/backend"));

        // Assert
        assert_eq!(
            class.unwrap(),
            Classification::ChildRoot(&repo(root, "backend"))
        );
    }

    #[test]
    fn classifies_paths_outside_every_child_repo_as_the_root_repo()
    {
        // Arrange: docs is not a repo, README.md is a root file
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        for (target, root_path) in [
            ("/vmr/README.md", "README.md"),
            ("/vmr/docs/guide.md", "docs/guide.md"),
            ("/vmr/.gitvmr", ".gitvmr")
        ]
        {
            // Act
            let class = router.classify(Path::new(target));

            // Assert
            assert_eq!(
                class.unwrap(),
                Classification::Member(
                    &Repo::root(root),
                    PathBuf::from(root_path)
                )
            );
        }
    }

    #[test]
    fn classifies_a_path_inside_an_undeclared_repo_relative_to_it()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        for (target, path) in
            [("/vmr/scratch/notes.md", "notes.md"), ("/vmr/scratch", ".")]
        {
            // Act
            let class = router.classify(Path::new(target));

            // Assert
            assert_eq!(class.unwrap(), Classification::Undeclared {
                name: "scratch",
                path: PathBuf::from(path)
            });
        }
    }

    #[test]
    fn classifying_a_path_inside_a_missing_child_repo_fails()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        // Act
        let err = router.classify(Path::new("/vmr/api/src")).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: 'api' is not cloned; run 'git vmr clone' to clone it"
        );
    }

    #[test]
    fn classifying_a_path_outside_the_vmr_fails()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        // Act
        let err = router.classify(Path::new("/elsewhere/a.rs")).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: '/elsewhere/a.rs' is outside virtual monorepo '/vmr'"
        );
    }

    #[test]
    fn routes_repo_root_to_dot()
    {
        // Arrange
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let backend = root.join("backend");

        // The aggregate path is denied however the user spells it
        for (working_dir, path) in [(root, "."), (backend.as_path(), "..")]
        {
            // Arrange
            let router = Router::new(root, &membership, working_dir);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        // Arrange: scratch is an undeclared repo, docs is not a repo,
        // README.md is a root file
        let root = Path::new("/vmr");
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

        for path in ["scratch/notes.md", "docs/readme.md", "README.md"]
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
        let membership = membership(root);
        let router = Router::new(root, &membership, root);

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
        let membership = membership(root);
        let working_dir = root.join("frontend");
        let router = Router::new(root, &membership, &working_dir);

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
        let membership = membership(root);
        let working_dir = root.join("frontend");
        let router = Router::new(root, &membership, &working_dir);

        // Act
        let target = router.target(Path::new("/vmr/backend/../backend/a.rs"));

        // Assert
        assert_eq!(target, PathBuf::from("/vmr/backend/a.rs"));
    }
}
