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
