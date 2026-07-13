use crate::git::{self, Head};
use crate::render::{SuffixPolicy, repo_list_suffix};
use crate::workspace::worktree_root::WorktreeRootEntry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Renders the worktree list: one block per worktree root, entries grouped
/// by state with a repo-list suffix when a state is not shared by all repos.
pub fn worktree_list(
    groups: BTreeMap<PathBuf, Vec<WorktreeRootEntry>>,
    repo_names: &[String]
) -> String
{
    let mut output = String::new();

    for (root, entries) in groups
    {
        render_group(&mut output, &root, entries, repo_names);
    }

    output
}

fn render_group(
    output: &mut String,
    root: &Path,
    entries: Vec<WorktreeRootEntry>,
    repo_names: &[String]
)
{
    let mut head_groups: BTreeMap<Head, Vec<String>> = BTreeMap::new();

    for entry in entries
    {
        head_groups.entry(entry.head).or_default().push(entry.repo);
    }

    let mut lines = Vec::new();
    for (head, mut repos) in head_groups
    {
        repos.sort();
        let repos = repos.iter().map(String::as_str).collect::<Vec<_>>();
        let suffix =
            repo_list_suffix(&repos, repo_names.len(), SuffixPolicy::Truncated);

        lines.push(format!("{}{}", render_head(&head), suffix));
    }

    if lines.len() == 1
    {
        output.push_str(&format!(
            "{} {}\n",
            git::git_style_path(root),
            lines[0]
        ));
    }
    else
    {
        output.push_str(&format!("{}\n", git::git_style_path(root)));
        for line in lines
        {
            output.push_str(&format!("  {line}\n"));
        }
    }
}

fn render_head(head: &Head) -> String
{
    match head
    {
        Head::Branch(branch) | Head::Unborn(branch) => format!("[{branch}]"),
        Head::Detached(hash) => format!("{hash} (detached HEAD)")
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn renders_empty_output_for_no_worktrees()
    {
        assert_eq!(worktree_list(BTreeMap::new(), &[]), "");
    }

    #[test]
    fn renders_shared_head_on_single_line_without_repo_list()
    {
        let groups = BTreeMap::from([(PathBuf::from("../release"), vec![
            branch_entry("frontend", "release"),
            branch_entry("backend", "release"),
        ])]);
        let repo_names = vec!["backend".to_owned(), "frontend".to_owned()];

        assert_eq!(
            worktree_list(groups, &repo_names),
            "../release [release]\n"
        );
    }

    #[test]
    fn groups_and_sorts_heads_in_multi_line_block()
    {
        let groups = BTreeMap::from([(PathBuf::from("../feature"), vec![
            detached_entry("tools", "fedcba98"),
            branch_entry("frontend", "topic"),
            detached_entry("backend", "01234567"),
            branch_entry("backend", "topic"),
            branch_entry("zeta", "alpha"),
            branch_entry("charlie", "alpha"),
            branch_entry("alpha", "alpha"),
            branch_entry("bravo", "alpha"),
        ])]);
        let repo_names = [
            "alpha", "backend", "bravo", "charlie", "frontend", "tools", "zeta"
        ]
        .map(str::to_owned);

        assert_eq!(
            worktree_list(groups, &repo_names),
            concat!(
                "../feature\n",
                "  [alpha] \x1b[90m(alpha, bravo, charlie, +1)\x1b[0m\n",
                "  [topic] \x1b[90m(backend, frontend)\x1b[0m\n",
                "  01234567 (detached HEAD) \x1b[90m(backend)\x1b[0m\n",
                "  fedcba98 (detached HEAD) \x1b[90m(tools)\x1b[0m\n"
            )
        );
    }

    fn branch_entry(repo: &str, branch: &str) -> WorktreeRootEntry
    {
        WorktreeRootEntry {
            repo: repo.to_owned(),
            head: Head::Branch(branch.to_owned())
        }
    }

    fn detached_entry(repo: &str, hash: &str) -> WorktreeRootEntry
    {
        WorktreeRootEntry {
            repo: repo.to_owned(),
            head: Head::Detached(hash.to_owned())
        }
    }
}
