use crate::git::{Head, RepoBranches};
use crate::workspace::Workspace;
use anstyle::{AnsiColor, Style};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct BranchStyle(Style);

impl BranchStyle
{
    fn render_text(self, text: &str) -> String
    {
        format!("{}{text}{}", self.0.render(), self.0.render_reset())
    }
}

struct BranchStyles
{
    active: BranchStyle,
    repo_list: BranchStyle
}

impl BranchStyles
{
    fn new() -> Self
    {
        Self {
            active: BranchStyle(
                Style::new().fg_color(Some(AnsiColor::Green.into()))
            ),
            repo_list: BranchStyle(
                Style::new().fg_color(Some(AnsiColor::BrightBlack.into()))
            )
        }
    }
}

pub fn branch(workspace: &Workspace, branch_name: &str) -> Result<()>
{
    // Branch in each child repository
    workspace.run(|git, repo| git.branch(&repo.name, &repo.path, branch_name))
}

pub fn delete(
    workspace: &Workspace,
    branch_name: &str,
    force: bool
) -> Result<()>
{
    // Delete branch in each child repository
    workspace.run(|git, repo| {
        git.delete_branch(&repo.name, &repo.path, branch_name, force)
    })
}

pub fn branches(workspace: &Workspace) -> Result<()>
{
    // Collect branch information from child repositories, in repo order
    let branches = workspace
        .map(|git, repo| git.branches(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Print results
    anstream::print!("{}", render_branches(&branches));
    Ok(())
}

fn render_branches(repos: &[(String, RepoBranches)]) -> String
{
    let mut output = String::new();
    let styles = BranchStyles::new();
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
            output.push_str(&styles.active.render_text(branch));
        }
        else
        {
            output.push_str(branch);
        }

        if branch_repos.len() != repo_count
        {
            let repo_names = branch_repos.into_iter().collect::<Vec<_>>();
            output.push(' ');
            output.push_str(
                &styles
                    .repo_list
                    .render_text(&format!("({})", repo_names.join(", ")))
            );
        }

        output.push('\n');
    }

    for (repo, hash) in detached
    {
        output.push_str(&format!("* (HEAD detached at {hash}) ({repo})\n"));
    }

    output
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn renders_shared_branch_without_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), repo_branches(["main"], "main")),
            ("frontend".to_owned(), repo_branches(["main"], "main")),
        ];

        let output = render_branches(&repos);

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

        let output = render_branches(&repos);

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

        let output = render_branches(&repos);

        assert!(output.contains("* \x1b[32mmain\x1b[0m\n"));
        assert!(output.contains("  topic\n"));
    }

    #[test]
    fn renders_detached_head_lines()
    {
        let repos = vec![
            ("backend".to_owned(), detached_repo(["main"], "a1b2c3d")),
            ("frontend".to_owned(), detached_repo(["main"], "d4e5f6a")),
        ];

        let output = render_branches(&repos);

        assert!(output.contains("  main\n"));
        assert!(output.contains("* (HEAD detached at a1b2c3d) (backend)\n"));
        assert!(output.contains("* (HEAD detached at d4e5f6a) (frontend)\n"));
    }

    #[test]
    fn renders_empty_output_for_no_repos_or_branch_refs()
    {
        assert_eq!(render_branches(&[]), "");

        let repos = vec![("backend".to_owned(), repo_branches([], "main"))];
        assert_eq!(render_branches(&repos), "");
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
}
