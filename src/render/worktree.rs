use crate::git::{self, ChildWorktreeState};
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
    let mut state_groups: BTreeMap<RenderedState, Vec<String>> =
        BTreeMap::new();

    for entry in entries
    {
        state_groups
            .entry(RenderedState::from_entry(&entry))
            .or_default()
            .push(entry.repo);
    }

    let mut lines = Vec::new();
    for (state, mut repos) in state_groups
    {
        repos.sort();
        let repos = repos.iter().map(String::as_str).collect::<Vec<_>>();
        let suffix =
            repo_list_suffix(&repos, repo_names.len(), SuffixPolicy::Truncated);

        lines.push(format!("{}{}", state.render(), suffix));
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

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum RenderedState
{
    Branch(String),
    Detached(String)
}

impl RenderedState
{
    fn from_entry(entry: &WorktreeRootEntry) -> Self
    {
        match &entry.state
        {
            ChildWorktreeState::Branch(branch) => Self::Branch(branch.clone()),
            ChildWorktreeState::Detached =>
                Self::Detached(short_head(&entry.head).to_owned()),
        }
    }

    fn render(&self) -> String
    {
        match self
        {
            Self::Branch(branch) => format!("[{branch}]"),
            Self::Detached(head) => format!("{head} (detached HEAD)")
        }
    }
}

fn short_head(head: &str) -> &str
{
    head.get(..8).unwrap_or(head)
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
    fn renders_shared_state_on_single_line_without_repo_list()
    {
        let groups = BTreeMap::from([(PathBuf::from("../release"), vec![
            branch_entry("frontend", "release", "22222222"),
            branch_entry("backend", "release", "11111111"),
        ])]);
        let repo_names = vec!["backend".to_owned(), "frontend".to_owned()];

        let output = worktree_list(groups, &repo_names);

        assert_eq!(output, "../release [release]\n");
    }

    #[test]
    fn groups_and_sorts_states_in_multi_line_block()
    {
        let groups = BTreeMap::from([(PathBuf::from("../feature"), vec![
            detached_entry("tools", "fedcba9876543210"),
            branch_entry("frontend", "topic", "33333333"),
            detached_entry("backend", "0123456789abcdef"),
            branch_entry("backend", "topic", "11111111"),
            branch_entry("tools", "alpha", "22222222"),
        ])]);
        let repo_names = vec![
            "backend".to_owned(),
            "frontend".to_owned(),
            "tools".to_owned(),
        ];

        let output = worktree_list(groups, &repo_names);

        assert_eq!(
            output,
            concat!(
                "../feature\n",
                "  [alpha] \x1b[90m(tools)\x1b[0m\n",
                "  [topic] \x1b[90m(backend, frontend)\x1b[0m\n",
                "  01234567 (detached HEAD) \x1b[90m(backend)\x1b[0m\n",
                "  fedcba98 (detached HEAD) \x1b[90m(tools)\x1b[0m\n"
            )
        );
    }

    fn branch_entry(repo: &str, branch: &str, head: &str) -> WorktreeRootEntry
    {
        WorktreeRootEntry {
            repo: repo.to_owned(),
            head: head.to_owned(),
            state: ChildWorktreeState::Branch(branch.to_owned())
        }
    }

    fn detached_entry(repo: &str, head: &str) -> WorktreeRootEntry
    {
        WorktreeRootEntry {
            repo: repo.to_owned(),
            head: head.to_owned(),
            state: ChildWorktreeState::Detached
        }
    }
}
