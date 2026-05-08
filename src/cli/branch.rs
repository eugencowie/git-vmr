use crate::cli::AggregateError;
use crate::git::{self, GitOutput, git_output, git_stdout};
use crate::vmr::{Repo, Vmr};
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Clone, PartialEq, Eq)]
enum Head
{
    Branch(String),
    Detached(String)
}

#[derive(Clone, PartialEq, Eq)]
struct RepoBranches
{
    branches: Vec<String>,
    head: Head
}

pub fn branch(working_dir: &Path, branch_name: Option<&str>) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;

    if let Some(branch_name) = branch_name
    {
        return create_branch(&vmr, branch_name);
    }

    let repos = collect_branches(&vmr)?;

    anstream::print!("{}", render_branches(&repos));

    Ok(())
}

fn collect_branches(vmr: &Vmr) -> Result<Vec<(String, RepoBranches)>>
{
    let repos = vmr.repos()?;

    let mut branches = repos
        .par_iter()
        .filter_map(|repo| collect_repo_branches(repo).transpose())
        .collect::<Result<Vec<_>>>()?;

    branches.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(branches)
}

fn collect_repo_branches(repo: &Repo)
-> Result<Option<(String, RepoBranches)>>
{
    let branches_output = git_stdout(&repo.path, [
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/heads"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo.path.display()
        )
    })?;
    let branches = String::from_utf8_lossy(&branches_output)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let head = match git_output(&repo.path, [
        "symbolic-ref",
        "--quiet",
        "--short",
        "HEAD"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo.path.display()
        )
    })?
    {
        GitOutput { status, stdout, stderr: _ } if status.success() =>
            Head::Branch(String::from_utf8_lossy(&stdout).trim().to_owned()),
        GitOutput { status, .. } if status.code() == Some(1) =>
        {
            let hash = String::from_utf8_lossy(
                &git_stdout(&repo.path, ["rev-parse", "--short", "HEAD"])
                    .with_context(|| {
                        format!(
                            "failed to read branch information for '{}'",
                            repo.path.display()
                        )
                    })?
            )
            .trim()
            .to_owned();
            Head::Detached(hash)
        }
        GitOutput { stderr, .. } => bail!(
            "failed to read branch information for '{}': {}",
            repo.path.display(),
            String::from_utf8_lossy(&stderr).trim()
        )
    };

    Ok(Some((repo.name.clone(), RepoBranches { branches, head })))
}

fn create_branch(vmr: &Vmr, branch_name: &str) -> Result<()>
{
    let repos = vmr.repos()?;
    let mut failures = repos
        .par_iter()
        .filter_map(|repo| {
            git::branch(&repo.name, &repo.path, branch_name).transpose()
        })
        .collect::<Result<Vec<_>>>()?;

    failures.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

    if !failures.is_empty()
    {
        return Err(AggregateError::new(
            failures
                .into_iter()
                .map(|failure| {
                    anyhow::anyhow!(
                        "{} ({})",
                        failure.message,
                        failure.repo_name
                    )
                })
                .collect()
        )
        .into());
    }

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
