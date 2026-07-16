use crate::cli::CliContext;
use crate::render::{ACTIVE, Rendered, SuffixPolicy, paint, repo_list_suffix};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: BranchArgs
) -> Result<Rendered>
{
    match args.action()
    {
        BranchAction::List => branches(workspace),

        // Branch in each child repository
        BranchAction::Create(branch_name) =>
            workspace.run(|git, repo| git.branch(repo, branch_name)),

        // Delete branch in each child repository
        BranchAction::Delete { branch_name, force } => workspace
            .run(|git, repo| git.delete_branch(repo, branch_name, force))
    }
}

fn branches(workspace: &Workspace) -> Result<Rendered>
{
    // Collect branch information from child repositories, in repo order
    let branches = workspace
        .map(|git, repo| git.branches(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render results
    Ok(render(&branches).into())
}

use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, Head};
use crate::vmr::Repo;
use anyhow::Context;

#[derive(Clone, PartialEq, Eq)]
struct RepoBranches
{
    branches: Vec<String>,
    head: Head
}

/// List, create, or delete branches
#[derive(clap::Args)]
pub struct BranchArgs
{
    /// Delete a branch. The branch must be fully merged in its upstream
    /// branch
    #[arg(
        short,
        long,
        conflicts_with = "force_delete",
        requires = "branch_name"
    )]
    pub delete: bool,

    /// Shortcut for `--delete --force`
    #[arg(short = 'D', conflicts_with = "delete", requires = "branch_name")]
    pub force_delete: bool,

    /// In combination with `-d` (or `--delete`), allow deleting the branch
    /// irrespective of its merged status, or whether it even points to a
    /// valid commit
    #[arg(
        short,
        long,
        conflicts_with = "force_delete",
        requires_all = ["branch_name", "delete"]
    )]
    pub force: bool,

    /// Creates a new branch head named [branch-name] which points to the
    /// current HEAD
    #[arg(value_name = "branch-name")]
    pub branch_name: Option<String>
}

/// What a `branch` invocation asks for. Total: the flag combinations the
/// arg attributes above reject cannot reach this enum.
pub enum BranchAction<'a>
{
    List,
    Create(&'a str),
    Delete
    {
        branch_name: &'a str,
        force: bool
    }
}

impl BranchArgs
{
    fn action(&self) -> BranchAction<'_>
    {
        match &self.branch_name
        {
            Some(branch_name) if self.delete || self.force_delete =>
                BranchAction::Delete {
                    branch_name,
                    force: self.force || self.force_delete
                },
            Some(branch_name) => BranchAction::Create(branch_name),
            None => BranchAction::List
        }
    }
}

impl Git
{
    fn branch(&self, repo: &Repo, branch_name: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["branch", branch_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrOnly, "git branch failed")
        )
    }

    fn branches(&self, repo: &Repo) -> Result<Option<(String, RepoBranches)>>
    {
        let branches_output = self
            .stdout(&repo.path, [
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/heads"
            ])
            .with_context(|| {
                format!(
                    "fatal: failed to read branch information for '{}'",
                    repo.path.display()
                )
            })?;
        let branches = String::from_utf8_lossy(&branches_output)
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let head = self.head(&repo.path).with_context(|| {
            format!(
                "fatal: failed to read branch information for '{}'",
                repo.path.display()
            )
        })?;

        Ok(Some((repo.name.clone(), RepoBranches { branches, head })))
    }
}

use std::collections::{BTreeMap, BTreeSet};

/// Renders the branch list grouped across child repos, marking active
/// branches and detached heads.
fn render(repos: &[(String, RepoBranches)]) -> String
{
    let mut output = String::new();
    let repo_count = repos.len();
    let mut branch_groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut active_branches = BTreeSet::new();
    let mut detached = Vec::new();

    for (repo, branches) in repos
    {
        for branch in &branches.branches
        {
            branch_groups
                .entry(branch.as_str())
                .or_default()
                .insert(repo.as_str());
        }

        match &branches.head
        {
            Head::Branch(branch) =>
            {
                active_branches.insert(branch.as_str());
            }
            Head::Unborn(branch) =>
            {
                branch_groups
                    .entry(branch.as_str())
                    .or_default()
                    .insert(repo.as_str());
                active_branches.insert(branch.as_str());
            }
            Head::Detached(hash) =>
            {
                detached.push((repo.as_str(), hash.as_str()));
            }
        }
    }

    detached.sort_by(|(repo_a, hash_a), (repo_b, hash_b)| {
        repo_a.cmp(repo_b).then(hash_a.cmp(hash_b))
    });

    for (branch, branch_repos) in branch_groups
    {
        let active = active_branches.contains(branch);
        let marker = if active { '*' } else { ' ' };
        output.push(marker);
        output.push(' ');
        if active
        {
            output.push_str(&paint(ACTIVE, branch));
        }
        else
        {
            output.push_str(branch);
        }

        let repo_names = branch_repos.into_iter().collect::<Vec<_>>();
        output.push_str(&repo_list_suffix(
            &repo_names,
            repo_count,
            SuffixPolicy::Truncated
        ));

        output.push('\n');
    }

    for (repo, hash) in detached
    {
        output.push_str(&format!(
            "* (HEAD detached at {hash}){}\n",
            repo_list_suffix(&[repo], repo_count, SuffixPolicy::Truncated)
        ));
    }

    output
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Head, ScriptedFake};
    use std::path::PathBuf;

    fn repo() -> Repo
    {
        Repo { name: "backend".to_owned(), path: PathBuf::from("/vmr/backend") }
    }

    #[test]
    fn branches_resolves_detached_head_through_rev_parse_fallback()
    {
        // Arrange
        let git = Git::with(
            ScriptedFake::new()
                .on(
                    ["for-each-ref", "--format=%(refname:short)", "refs/heads"],
                    0,
                    "develop\nmain\n",
                    ""
                )
                .on(["symbolic-ref", "--quiet", "--short", "HEAD"], 1, "", "")
                .on(["rev-parse", "--short=8", "HEAD"], 0, "abc12345\n", "")
        );

        // Act
        let (name, branches) = git.branches(&repo()).unwrap().unwrap();

        // Assert
        assert_eq!(name, "backend");
        assert_eq!(branches.branches, vec!["develop", "main"]);
        assert!(
            matches!(&branches.head, Head::Detached(hash) if hash == "abc12345")
        );
    }
}

#[cfg(test)]
mod render_tests
{
    use super::*;

    #[test]
    fn renders_shared_branch_without_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), repo_branches(["main"], "main")),
            ("frontend".to_owned(), repo_branches(["main"], "main")),
        ];

        let output = render(&repos);

        assert_eq!(output, "* \x1b[32mmain\x1b[0m\n");
    }

    #[test]
    fn renders_partial_branch_with_sorted_repo_list()
    {
        let repos = vec![
            (
                "backend".to_owned(),
                repo_branches(["main", "release/1.2"], "main")
            ),
            (
                "frontend".to_owned(),
                repo_branches(
                    ["feature/auth", "main", "release/1.2"],
                    "feature/auth"
                )
            ),
            ("tools".to_owned(), repo_branches(["main"], "main")),
        ];

        let output = render(&repos);

        assert!(output.contains(
            "* \x1b[32mfeature/auth\x1b[0m \x1b[90m(frontend)\x1b[0m\n"
        ));
        assert!(output.contains("* \x1b[32mmain\x1b[0m\n"));
        assert!(
            output
                .contains("  release/1.2 \x1b[90m(backend, frontend)\x1b[0m\n")
        );
    }

    #[test]
    fn renders_inactive_marker_for_branch_not_checked_out()
    {
        let repos = vec![(
            "backend".to_owned(),
            repo_branches(["main", "topic"], "main")
        )];

        let output = render(&repos);

        assert!(output.contains("* \x1b[32mmain\x1b[0m\n"));
        assert!(output.contains("  topic\n"));
    }

    #[test]
    fn renders_unborn_head_as_an_active_branch()
    {
        let repos = vec![
            ("backend".to_owned(), unborn_repo([], "main")),
            ("frontend".to_owned(), repo_branches(["main"], "main")),
        ];

        let output = render(&repos);

        assert_eq!(output, "* \x1b[32mmain\x1b[0m\n");
    }

    #[test]
    fn renders_detached_head_lines()
    {
        let repos = vec![
            ("backend".to_owned(), detached_repo(["main"], "a1b2c3d")),
            ("frontend".to_owned(), detached_repo(["main"], "d4e5f6a")),
        ];

        let output = render(&repos);

        assert!(output.contains("  main\n"));
        assert!(output.contains(
            "* (HEAD detached at a1b2c3d) \x1b[90m(backend)\x1b[0m\n"
        ));
        assert!(output.contains(
            "* (HEAD detached at d4e5f6a) \x1b[90m(frontend)\x1b[0m\n"
        ));
    }

    #[test]
    fn renders_detached_head_line_without_repo_list_for_single_repo()
    {
        let repos =
            vec![("backend".to_owned(), detached_repo(["main"], "a1b2c3d"))];

        let output = render(&repos);

        assert!(output.contains("* (HEAD detached at a1b2c3d)\n"));
    }

    #[test]
    fn renders_empty_output_for_no_repos_or_branch_refs()
    {
        assert_eq!(render(&[]), "");

        let repos = vec![("backend".to_owned(), repo_branches([], "main"))];
        assert_eq!(render(&repos), "");
    }

    fn repo_branches<const N: usize>(
        branches: [&str; N],
        active: &str
    ) -> RepoBranches
    {
        RepoBranches {
            branches: branches
                .iter()
                .map(|branch| (*branch).to_owned())
                .collect(),
            head: Head::Branch(active.to_owned())
        }
    }

    fn detached_repo<const N: usize>(
        branches: [&str; N],
        hash: &str
    ) -> RepoBranches
    {
        RepoBranches {
            branches: branches
                .iter()
                .map(|branch| (*branch).to_owned())
                .collect(),
            head: Head::Detached(hash.to_owned())
        }
    }

    fn unborn_repo<const N: usize>(
        branches: [&str; N],
        active: &str
    ) -> RepoBranches
    {
        RepoBranches {
            branches: branches
                .iter()
                .map(|branch| (*branch).to_owned())
                .collect(),
            head: Head::Unborn(active.to_owned())
        }
    }
}
