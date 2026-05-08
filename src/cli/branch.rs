use crate::cli::AggregateError;
use crate::config::vmr;
use crate::git::{self, GitOutput, git_output, git_stdout};
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

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
    let vmr_root = vmr::find_vmr_root(working_dir)?;

    if let Some(branch_name) = branch_name
    {
        return create_branch(&vmr_root, branch_name);
    }

    let repos = collect_branches(&vmr_root)?;

    anstream::print!("{}", render_branches(&repos));

    Ok(())
}

fn child_dirs(vmr_root: &Path) -> Result<Vec<PathBuf>>
{
    Ok(fs::read_dir(vmr_root)
        .with_context(|| {
            format!("failed to read VMR root '{}'", vmr_root.display())
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false)
        })
        .filter(|entry| entry.file_name() != OsStr::new(".gitvmr"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>())
}

fn collect_branches(vmr_root: &Path) -> Result<Vec<(String, RepoBranches)>>
{
    let children = child_dirs(vmr_root)?;

    let mut repos = children
        .par_iter()
        .filter_map(|path| collect_repo_branches(path).transpose())
        .collect::<Result<Vec<_>>>()?;

    repos.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(repos)
}

fn collect_repo_branches(
    repo_path: &Path
) -> Result<Option<(String, RepoBranches)>>
{
    if !repo_path.join(".git").exists()
    {
        return Ok(None);
    }

    let repo_name = repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("repository path has no valid UTF-8 file name")?
        .to_owned();

    let branches_output = git_stdout(repo_path, &[
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/heads"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo_path.display()
        )
    })?;
    let branches = String::from_utf8_lossy(&branches_output)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let head = match git_output(repo_path, &[
        "symbolic-ref",
        "--quiet",
        "--short",
        "HEAD"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo_path.display()
        )
    })?
    {
        GitOutput { status, stdout, stderr: _ } if status.success() =>
            Head::Branch(String::from_utf8_lossy(&stdout).trim().to_owned()),
        GitOutput { status, .. } if status.code() == Some(1) =>
        {
            let hash = String::from_utf8_lossy(
                &git_stdout(repo_path, &["rev-parse", "--short", "HEAD"])
                    .with_context(|| {
                        format!(
                            "failed to read branch information for '{}'",
                            repo_path.display()
                        )
                    })?
            )
            .trim()
            .to_owned();
            Head::Detached(hash)
        }
        GitOutput { stderr, .. } => bail!(
            "failed to read branch information for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&stderr).trim()
        )
    };

    Ok(Some((repo_name, RepoBranches { branches, head })))
}

fn create_branch(vmr_root: &Path, branch_name: &str) -> Result<()>
{
    let repos = eligible_repos(vmr_root)?;
    let mut failures = repos
        .par_iter()
        .filter_map(|(repo_name, repo_path)| {
            git::branch(repo_name, repo_path, branch_name).transpose()
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

fn eligible_repos(vmr_root: &Path) -> Result<Vec<(String, PathBuf)>>
{
    let mut repos = Vec::new();

    for repo_path in child_dirs(vmr_root)?
    {
        if !repo_path.join(".git").exists()
        {
            continue;
        }

        let repo_name = repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .context("repository path has no valid UTF-8 file name")?
            .to_owned();

        repos.push((repo_name, repo_path));
    }

    repos.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(repos)
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
