use crate::git::{Head, RepoBranches};
use crate::render::{ACTIVE, SuffixPolicy, paint, repo_list_suffix};
use std::collections::{BTreeMap, BTreeSet};

/// Renders the branch list grouped across child repos, marking active
/// branches and detached heads.
pub fn branches(repos: &[(String, RepoBranches)]) -> String
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

    #[test]
    fn renders_shared_branch_without_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), repo_branches(["main"], "main")),
            ("frontend".to_owned(), repo_branches(["main"], "main")),
        ];

        let output = branches(&repos);

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

        let output = branches(&repos);

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

        let output = branches(&repos);

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

        let output = branches(&repos);

        assert_eq!(output, "* \x1b[32mmain\x1b[0m\n");
    }

    #[test]
    fn renders_detached_head_lines()
    {
        let repos = vec![
            ("backend".to_owned(), detached_repo(["main"], "a1b2c3d")),
            ("frontend".to_owned(), detached_repo(["main"], "d4e5f6a")),
        ];

        let output = branches(&repos);

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

        let output = branches(&repos);

        assert!(output.contains("* (HEAD detached at a1b2c3d)\n"));
    }

    #[test]
    fn renders_empty_output_for_no_repos_or_branch_refs()
    {
        assert_eq!(branches(&[]), "");

        let repos = vec![("backend".to_owned(), repo_branches([], "main"))];
        assert_eq!(branches(&repos), "");
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
