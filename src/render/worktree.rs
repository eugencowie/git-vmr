use crate::git::{self, ChildWorktreeState};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One child repo's entry under a worktree root, as gathered for listing.
#[derive(Clone, Eq, PartialEq)]
pub struct WorktreeRootEntry
{
    pub repo: String,
    pub head: String,
    pub state: ChildWorktreeState
}

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
    mut entries: Vec<WorktreeRootEntry>,
    repo_names: &[String]
)
{
    entries.sort_by(|a, b| {
        (state_sort_key(&a.state), short_head(&a.head), &a.repo).cmp(&(
            state_sort_key(&b.state),
            short_head(&b.head),
            &b.repo
        ))
    });

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
        let suffix = if repos == repo_names
        {
            String::new()
        }
        else
        {
            format!(" ({})", repos.join(", "))
        };

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

fn state_sort_key(state: &ChildWorktreeState) -> (&str, &str)
{
    match state
    {
        ChildWorktreeState::Branch(branch) => ("branch", branch),
        ChildWorktreeState::Detached => ("detached", "")
    }
}
