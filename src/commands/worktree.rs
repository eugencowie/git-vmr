use crate::cli::CliContext;
use crate::render::{Rendered, SuffixPolicy, fail, outcomes, repo_list_suffix};
use crate::workspace::{
    Workspace, WorktreeAddArgs, WorktreeCommand, WorktreeMoveArgs,
    WorktreeRemoveArgs, resolve_target
};
use anyhow::Result;
use std::path::Path;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    command: Option<WorktreeCommand>
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    match command.unwrap_or(WorktreeCommand::List)
    {
        WorktreeCommand::Add(args) => add(workspace, working_dir, &args),
        WorktreeCommand::List => list(workspace),
        WorktreeCommand::Move(args) =>
            move_worktree(workspace, working_dir, &args),
        WorktreeCommand::Remove(args) => remove(workspace, working_dir, &args)
    }
}

fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let groups = workspace.worktree_roots().list()?;

    Ok(render(groups, &repo_names).into())
}

fn add(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeAddArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    outcomes(workspace.worktree_roots().add(
        &target,
        args.branch.as_deref(),
        args.commit_ish.as_deref()
    )?)
}

fn remove(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeRemoveArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    let removal = workspace.worktree_roots().remove(
        &target,
        args.force,
        args.delete,
        args.force_delete
    )?;

    let rendered = outcomes(removal.outcomes)?;

    // Keep the per-repo successes if dissolving the root fails
    match removal.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(fail(rendered, format!("{error:#}")))
    }
}

fn move_worktree(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeMoveArgs
) -> Result<Rendered>
{
    let source = resolve_target(working_dir, &args.path);
    let destination = resolve_target(working_dir, &args.new_path);

    let moved = workspace.worktree_roots().move_root(
        &source,
        &destination,
        args.force
    )?;

    let rendered = outcomes(moved.outcomes)?;

    // Keep the per-repo successes if dissolving the source root fails
    match moved.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(fail(rendered, format!("{error:#}")))
    }
}

use crate::git::{self, Head};
use crate::workspace::worktree_root::WorktreeRootEntry;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Renders the worktree list: one block per worktree root, entries grouped
/// by state with a repo-list suffix when a state is not shared by all repos.
fn render(
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
mod render_tests
{
    use super::*;

    #[test]
    fn renders_empty_output_for_no_worktrees()
    {
        assert_eq!(render(BTreeMap::new(), &[]), "");
    }

    #[test]
    fn renders_shared_head_on_single_line_without_repo_list()
    {
        let groups = BTreeMap::from([(PathBuf::from("../release"), vec![
            branch_entry("frontend", "release"),
            branch_entry("backend", "release"),
        ])]);
        let repo_names = vec!["backend".to_owned(), "frontend".to_owned()];

        assert_eq!(render(groups, &repo_names), "../release [release]\n");
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
            render(groups, &repo_names),
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
