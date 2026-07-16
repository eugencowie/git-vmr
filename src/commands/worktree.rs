mod roots;

use crate::cli::CliContext;
use crate::render::{Rendered, SuffixPolicy, fail, outcomes, repo_list_suffix};
use crate::workspace::{Workspace, resolve_target};
use anyhow::Result;
use clap::ArgAction;
use roots::WorktreeRoots;
use std::path::Path;

#[derive(clap::Subcommand)]
pub enum WorktreeCommand
{
    /// Create a worktree at <path> and checkout [commit-ish] into it
    Add(WorktreeAddArgs),

    /// List details of each worktree
    List,

    /// Move a worktree to a new location
    Move(WorktreeMoveArgs),

    /// Remove a worktree
    #[command(visible_alias = "rm")]
    Remove(WorktreeRemoveArgs)
}

#[derive(clap::Args)]
pub struct WorktreeAddArgs
{
    /// With add, create a new branch named <new-branch> starting at
    /// [commit-ish], and check out <new-branch> into the new worktree
    #[arg(short, value_name = "new-branch")]
    pub branch: Option<String>,

    #[arg(value_name = "path")]
    pub path: PathBuf,

    #[arg(value_name = "commit-ish")]
    pub commit_ish: Option<String>
}

#[derive(clap::Args)]
pub struct WorktreeMoveArgs
{
    /// Move a worktree even when Git would otherwise refuse. Specify twice
    /// for cases that require two force flags.
    #[arg(short, long, action = ArgAction::Count)]
    pub force: u8,

    /// Worktrees can be identified by path, either relative or absolute
    #[arg(value_name = "worktree")]
    pub path: PathBuf,

    /// New location for the worktree
    #[arg(value_name = "new-path")]
    pub new_path: PathBuf
}

#[derive(clap::Args)]
pub struct WorktreeRemoveArgs
{
    /// By default, remove refuses to remove an unclean worktree unless
    /// --force is used. To remove a locked worktree, specify --force twice
    #[arg(short, long, action = ArgAction::Count)]
    pub force: u8,

    /// Delete the branch
    #[arg(short, long, conflicts_with = "force_delete")]
    pub delete: bool,

    /// Force-delete the branch
    #[arg(short = 'D')]
    pub force_delete: bool,

    /// Worktrees can be identified by path, either relative or absolute
    #[arg(value_name = "worktree")]
    pub path: PathBuf
}

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    command: &Option<WorktreeCommand>
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    match command.as_ref().unwrap_or(&WorktreeCommand::List)
    {
        WorktreeCommand::Add(args) => add(workspace, working_dir, args),
        WorktreeCommand::List => list(workspace),
        WorktreeCommand::Move(args) => mv(workspace, working_dir, args),
        WorktreeCommand::Remove(args) => remove(workspace, working_dir, args)
    }
}

fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let groups = WorktreeRoots::new(workspace).list()?;

    Ok(render(groups, &repo_names).into())
}

fn add(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeAddArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    outcomes(
        WorktreeRoots::new(workspace).add(
            &target,
            args.branch.as_deref(),
            args.commit_ish.as_deref()
        )?,
        repo_scope(workspace)
    )
}

/// The scope of a root operation: one child worktree per child repo.
fn repo_scope<'a>(workspace: &'a Workspace) -> impl Iterator<Item = &'a str>
{
    workspace.repos().iter().map(|repo| repo.name.as_str())
}

fn remove(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeRemoveArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    let removal = WorktreeRoots::new(workspace).remove(
        &target,
        args.force,
        args.delete,
        args.force_delete
    )?;

    let rendered = outcomes(removal.outcomes, repo_scope(workspace))?;

    // Keep the per-repo successes if dissolving the root fails
    match removal.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(fail(rendered, format!("{error:#}")))
    }
}

fn mv(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeMoveArgs
) -> Result<Rendered>
{
    let source = resolve_target(working_dir, &args.path);
    let destination = resolve_target(working_dir, &args.new_path);

    let moved = WorktreeRoots::new(workspace).move_root(
        &source,
        &destination,
        args.force
    )?;

    let rendered = outcomes(moved.outcomes, repo_scope(workspace))?;

    // Keep the per-repo successes if dissolving the source root fails
    match moved.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(fail(rendered, format!("{error:#}")))
    }
}

use crate::git::{self, Head};
use roots::WorktreeRootEntry;
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

#[cfg(test)]
mod parse_tests
{
    use super::{
        WorktreeAddArgs, WorktreeCommand, WorktreeMoveArgs, WorktreeRemoveArgs
    };
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use clap::error::ErrorKind;

    #[test]
    fn parses_worktree_add_target_path()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "add", "../wt"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Add(WorktreeAddArgs {
                    branch: None,
                    path,
                    commit_ish: None
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_optional_commit_ish()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "add", "../wt", "main"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Add(WorktreeAddArgs {
                    branch: None,
                    path,
                    commit_ish: Some(commit_ish)
                }))
            }) if &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn parses_worktree_add_branch()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "worktree",
            "add",
            "-b",
            "feature/auth",
            "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Add(WorktreeAddArgs {
                    branch: Some(branch),
                    path,
                    commit_ish: None
                }))
            }) if branch == "feature/auth" && &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_branch_with_commit_ish()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "worktree",
            "add",
            "-b",
            "feature/auth",
            "../wt",
            "main"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Add(WorktreeAddArgs {
                    branch: Some(branch),
                    path,
                    commit_ish: Some(commit_ish)
                }))
            }) if branch == "feature/auth" && &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn rejects_worktree_add_branch_without_value()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "add", "-b"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    #[test]
    fn rejects_worktree_add_long_branch_alias()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr",
            "worktree",
            "add",
            "--branch",
            "feature/auth",
            "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_add_without_target_path()
    {
        // Act
        let err =
            Cli::parse_from(["git-vmr", "worktree", "add"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_add_extra_operands()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr", "worktree", "add", "../wt", "main", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn parses_worktree_list()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "worktree", "list"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::List)
            })
        ));
    }

    #[test]
    fn parses_worktree_without_subcommand_as_default_list()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "worktree"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree { command: None })
        ));
    }

    #[test]
    fn rejects_worktree_list_extra_operands()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "list", "extra"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_list_options()
    {
        for option in ["--porcelain", "-z", "-v"]
        {
            // Act
            let err = Cli::parse_from(["git-vmr", "worktree", "list", option])
                .err()
                .unwrap();

            // Assert
            assert_eq!(err.kind(), ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn parses_worktree_remove_target_path()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "worktree", "remove", "../wt"])
            .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 0,
                    delete: false,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_rm_alias_target_path()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "rm", "../wt"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 0,
                    delete: false,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_single_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 1,
                    delete: false,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_long_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 2,
                    delete: false,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_short_force()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-ff", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 2,
                    delete: false,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_short_delete()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-d", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 0,
                    delete: true,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_long_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--delete", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 0,
                    delete: true,
                    force_delete: false,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_force_delete()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-D", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 0,
                    delete: false,
                    force_delete: true,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_force_and_force_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "-D", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 1,
                    delete: false,
                    force_delete: true,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_rm_alias_force_and_force_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "rm", "--force", "-D", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Remove(WorktreeRemoveArgs {
                    force: 1,
                    delete: false,
                    force_delete: true,
                    path
                }))
            }) if &path == "../wt"
        ));
    }

    #[test]
    fn rejects_worktree_remove_conflicting_delete_modes()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr", "worktree", "remove", "-d", "-D", "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rejects_worktree_remove_long_force_delete()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr",
            "worktree",
            "remove",
            "--force-delete",
            "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_remove_without_target_path()
    {
        // Act
        let err =
            Cli::parse_from(["git-vmr", "worktree", "remove"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_remove_extra_operands()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr", "worktree", "remove", "../wt", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn parses_worktree_move_paths()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Move(WorktreeMoveArgs {
                    force: 0,
                    path,
                    new_path
                }))
            }) if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_single_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "--force", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Move(WorktreeMoveArgs {
                    force: 1,
                    path,
                    new_path
                }))
            }) if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_long_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "--force", "--force", "../wt",
            "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Move(WorktreeMoveArgs {
                    force: 2,
                    path,
                    new_path
                }))
            }) if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_short_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "-ff", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Worktree {
                command: Some(WorktreeCommand::Move(WorktreeMoveArgs {
                    force: 2,
                    path,
                    new_path
                }))
            }) if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn rejects_worktree_move_without_source_path()
    {
        // Act
        let err =
            Cli::parse_from(["git-vmr", "worktree", "move"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_without_destination_path()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "move", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_extra_operands()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr", "worktree", "move", "../wt", "../moved", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_unsupported_worktree_subcommands()
    {
        // Act
        Cli::parse_from(["git-vmr", "worktree", "list"]).unwrap();
        let err = Cli::parse_from(["git-vmr", "worktree", "lock", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }
}
