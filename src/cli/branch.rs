use crate::cli::print_results;
use crate::git;
use crate::vmr::{Head, RepoBranches, Vmr};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn branch(working_dir: &Path, branch_name: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Branch in each repository
    let results = repos
        .par_iter()
        .map(|repo| git::branch(&repo.name, &repo.path, branch_name))
        .collect::<Vec<_>>();

    // Print results
    print_results(results)
}

pub fn branches(working_dir: &Path) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Collect branch information from repositories
    let mut branches = repos
        .par_iter()
        .filter_map(|repo| repo.branches().transpose())
        .collect::<Result<Vec<_>>>()?;

    // Keep branch order deterministic
    branches.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Print results
    anstream::print!("{}", render_branches(&branches));
    Ok(())
}

fn render_branches(repos: &[(String, RepoBranches)]) -> String
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
        let marker = if active_branches.contains(branch) { '*' } else { ' ' };
        output.push(marker);
        output.push(' ');
        output.push_str(branch);

        if branch_repos.len() != repo_count
        {
            let repo_names = branch_repos.into_iter().collect::<Vec<_>>();
            output.push_str(" (");
            output.push_str(&repo_names.join(", "));
            output.push(')');
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

        assert_eq!(output, "* main\n");
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

        assert!(output.contains("* feature/auth (frontend)\n"));
        assert!(output.contains("* main\n"));
        assert!(output.contains("  release/1.2 (backend, frontend)\n"));
    }

    #[test]
    fn renders_inactive_marker_for_branch_not_checked_out()
    {
        let repos = vec![(
            "backend".to_owned(),
            repo_branches(["main", "topic"], "main")
        )];

        let output = render_branches(&repos);

        assert!(output.contains("* main\n"));
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
