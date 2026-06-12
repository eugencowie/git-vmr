mod add;
mod branch;
mod clone;
mod commit;
mod fetch;
mod foreach;
mod init;
mod merge;
mod mv;
mod pull;
mod push;
mod rebase;
mod reset;
mod restore;
mod rm;
mod status;
mod switch;
mod tag;
mod worktree;

use crate::cli::CliContext;
use crate::git::{ChmodMode, ResetMode};
use anyhow::Result;
use clap::{ArgAction, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Command
{
    /// Clone a repository into a new directory
    Clone
    {
        /// The (possibly remote) <repository> to clone from
        #[arg(value_name = "repository")]
        repository: String,

        /// The name of a new directory to clone into
        #[arg(value_name = "directory")]
        directory: Option<PathBuf>
    },

    /// Create an empty virtual monorepo or reinitialize an existing one
    Init
    {
        /// If you provide a directory, the command is run inside it. If this
        /// directory does not exist, it will be created
        #[arg(value_name = "directory")]
        directory: Option<PathBuf>
    },

    /// Add file contents to the index
    Add
    {
        /// Allow adding otherwise ignored files
        #[arg(short, long)]
        force: bool,

        /// Update the index not only where the working tree has a file
        /// matching [pathspec] but also where the index already has an
        /// entry
        #[arg(short = 'A', long)]
        all: bool,

        /// Override the executable bit of added files
        #[arg(long, value_name = "(+|-)x")]
        chmod: Option<ChmodMode>,

        /// Files to add content from
        #[arg(required_unless_present = "all", num_args = 0.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Move or rename a file, a directory, or a symlink
    Mv
    {
        /// Files to move
        #[arg(required = true, num_args = 1.., value_name = "source")]
        sources: Vec<PathBuf>,

        /// Destination path
        #[arg(value_name = "destination")]
        destination: PathBuf
    },

    /// Restore working tree files
    Restore
    {
        /// Restore the working tree
        #[arg(long)]
        worktree: bool,

        /// Restore the index
        #[arg(long)]
        staged: bool,

        /// Files to restore
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Remove files from the working tree and from the index
    Rm
    {
        /// Allow recursive removal when a leading directory name is given
        #[arg(short)]
        recursive: bool,

        /// Override the up-to-date check
        #[arg(short, long)]
        force: bool,

        /// Don't actually remove any files
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Unstage and remove paths only from the index
        #[arg(long)]
        cached: bool,

        /// Files to remove
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Show the working tree status
    Status,

    /// List, create, or delete branches
    Branch
    {
        /// Delete a branch. The branch must be fully merged in its upstream
        /// branch
        #[arg(
            short,
            long,
            conflicts_with = "force_delete",
            requires = "branch_name"
        )]
        delete: bool,

        /// Shortcut for `--delete --force`
        #[arg(
            short = 'D',
            conflicts_with = "delete",
            requires = "branch_name"
        )]
        force_delete: bool,

        /// In combination with `-d` (or `--delete`), allow deleting the branch
        /// irrespective of its merged status, or whether it even points to a
        /// valid commit
        #[arg(
            short,
            long,
            conflicts_with = "force_delete",
            requires_all = ["branch_name", "delete"]
        )]
        force: bool,

        /// Creates a new branch head named [branch-name] which points to the
        /// current HEAD
        #[arg(value_name = "branch-name")]
        branch_name: Option<String>
    },

    /// Record changes to the repositories
    Commit
    {
        /// Use <msg> as the commit message
        #[arg(short, long, required = true, value_name = "msg")]
        message: String
    },

    /// Join two or more development histories together
    Merge
    {
        /// Commits, usually other branch heads, to merge into our branch
        #[arg(required = true, value_name = "commit")]
        commit_ish: String
    },

    /// Reapply commits on top of another base tip
    Rebase
    {
        /// Upstream branch to compare against
        #[arg(required = true, value_name = "upstream")]
        upstream: String
    },

    /// Set `HEAD` or the index to a known state
    Reset
    {
        /// Leave your working directory unchanged
        #[arg(long, conflicts_with_all = ["soft", "hard", "merge", "keep"])]
        mixed: bool,

        /// Leave your working tree files and the index unchanged
        #[arg(long, conflicts_with_all = ["mixed", "hard", "merge", "keep"])]
        soft: bool,

        /// Overwrite all files and directories with the version from [commit],
        /// and may overwrite untracked files
        #[arg(long, conflicts_with_all = ["soft", "mixed", "merge", "keep"])]
        hard: bool,

        /// Reset the index and update the files in the working tree that are
        /// different between [commit] and HEAD, but keep those which are
        /// different between the index and working tree (i.e. which have
        /// changes which have not been added)
        #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "keep"])]
        merge: bool,

        /// Resets index entries and updates files in the working tree that are
        /// different between [commit] and HEAD
        #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "merge"])]
        keep: bool,

        /// Set the current branch head (HEAD) to point at [commit]
        #[arg(value_name = "commit")]
        commit: Option<String>
    },

    /// Switch branches
    Switch
    {
        /// Create a new branch named <branch> before switching to the branch
        #[arg(short, long)]
        create: bool,

        /// Branch to switch to
        #[arg(required = true, value_name = "branch")]
        branch_name: String
    },

    /// Create, list, delete or verify tags
    Tag
    {
        /// Delete existing tags with the given names
        #[arg(short, long, requires = "tag_name")]
        delete: bool,

        /// The name of the tag to create, delete, or describe
        #[arg(value_name = "tagname")]
        tag_name: Option<String>
    },

    /// Download objects and refs from another repository
    Fetch
    {
        /// The "remote" repository that is the source of a fetch or pull
        /// operation
        #[arg(value_name = "repository")]
        repository: Option<String>,

        /// Specifies which refs to fetch and which local refs to update
        #[arg(value_name = "refspec")]
        refspecs: Vec<String>
    },

    /// Fetch from and integrate with another repository or a local branch
    Pull
    {
        /// The "remote" repository to pull from
        #[arg(value_name = "repository")]
        repository: Option<String>,

        /// Which branch or other reference(s) to fetch and integrate into the
        /// current branch
        #[arg(value_name = "refspec")]
        refspecs: Vec<String>
    },

    /// Update remote refs along with associated objects
    Push
    {
        /// The "remote" repository that is the destination of a push operation
        #[arg(value_name = "repository")]
        repository: Option<String>,

        /// Specify what destination ref to update with what source object
        #[arg(value_name = "refspec")]
        refspecs: Vec<String>
    },

    /// Manage multiple working trees
    Worktree
    {
        #[command(subcommand)]
        command: Option<WorktreeCommand>
    },

    /// Evaluates an arbitrary shell command in each checked out repository
    Foreach
    {
        /// Only print error messages
        #[arg(short, long)]
        quiet: bool,

        /// Command to evaluate through the shell
        #[arg(
                required = true,
                num_args = 1..,
                trailing_var_arg = true,
                allow_hyphen_values = true,
                value_name = "command"
            )]
        command: Vec<String>
    }
}

#[derive(Subcommand)]
pub enum WorktreeCommand
{
    /// Create a worktree at <path> and checkout [commit-ish] into it
    Add
    {
        /// With add, create a new branch named <new-branch> starting at
        /// [commit-ish], and check out <new-branch> into the new worktree
        #[arg(short, value_name = "new-branch")]
        branch: Option<String>,

        #[arg(value_name = "path")]
        path: PathBuf,

        #[arg(value_name = "commit-ish")]
        commit_ish: Option<String>
    },

    /// List details of each worktree
    List,

    /// Move a worktree to a new location
    Move
    {
        /// Move a worktree even when Git would otherwise refuse. Specify twice
        /// for cases that require two force flags.
        #[arg(short, long, action = ArgAction::Count)]
        force: u8,

        /// Worktrees can be identified by path, either relative or absolute
        #[arg(value_name = "worktree")]
        path: PathBuf,

        /// New location for the worktree
        #[arg(value_name = "new-path")]
        new_path: PathBuf
    },

    /// Remove a worktree
    #[command(visible_alias = "rm")]
    Remove
    {
        /// By default, remove refuses to remove an unclean worktree unless
        /// --force is used. To remove a locked worktree, specify --force twice
        #[arg(short, long, action = ArgAction::Count)]
        force: u8,

        /// Delete the branch
        #[arg(short, long, conflicts_with = "force_delete")]
        delete: bool,

        /// Force-delete the branch
        #[arg(short = 'D')]
        force_delete: bool,

        /// Worktrees can be identified by path, either relative or absolute
        #[arg(value_name = "worktree")]
        path: PathBuf
    }
}

impl Command
{
    pub fn command_name(&self) -> &'static str
    {
        match self
        {
            Self::Clone { .. } => "clone",
            Self::Init { .. } => "init",
            Self::Add { .. } => "add",
            Self::Mv { .. } => "mv",
            Self::Restore { .. } => "restore",
            Self::Rm { .. } => "rm",
            Self::Status => "status",
            Self::Branch { .. } => "branch",
            Self::Commit { .. } => "commit",
            Self::Merge { .. } => "merge",
            Self::Rebase { .. } => "rebase",
            Self::Reset { .. } => "reset",
            Self::Switch { .. } => "switch",
            Self::Tag { .. } => "tag",
            Self::Fetch { .. } => "fetch",
            Self::Pull { .. } => "pull",
            Self::Push { .. } => "push",
            Self::Worktree { command } => match command
            {
                Some(WorktreeCommand::Add { .. }) => "worktree.add",
                Some(WorktreeCommand::List) | None => "worktree.list",
                Some(WorktreeCommand::Move { .. }) => "worktree.move",
                Some(WorktreeCommand::Remove { .. }) => "worktree.remove"
            },
            Self::Foreach { .. } => "foreach"
        }
    }

    pub fn flag_names(&self) -> Vec<&'static str>
    {
        let mut flags = Vec::new();

        match self
        {
            Self::Add { force, all, chmod, .. } =>
            {
                push_if(&mut flags, *force, "force");
                push_if(&mut flags, *all, "all");
                push_if(&mut flags, chmod.is_some(), "chmod");
            }
            Self::Restore { worktree, staged, .. } =>
            {
                push_if(&mut flags, *worktree, "worktree");
                push_if(&mut flags, *staged, "staged");
            }
            Self::Rm { recursive, force, dry_run, cached, .. } =>
            {
                push_if(&mut flags, *recursive, "recursive");
                push_if(&mut flags, *force, "force");
                push_if(&mut flags, *dry_run, "dry_run");
                push_if(&mut flags, *cached, "cached");
            }
            Self::Branch { delete, force_delete, force, .. } =>
            {
                push_if(&mut flags, *delete, "delete");
                push_if(&mut flags, *force_delete, "force_delete");
                push_if(&mut flags, *force, "force");
            }
            Self::Commit { .. } => flags.push("message"),
            Self::Reset { soft, mixed, hard, merge, keep, .. } =>
            {
                push_if(&mut flags, *soft, "soft");
                push_if(&mut flags, *mixed, "mixed");
                push_if(&mut flags, *hard, "hard");
                push_if(&mut flags, *merge, "merge");
                push_if(&mut flags, *keep, "keep");
            }
            Self::Switch { create, .. } =>
                push_if(&mut flags, *create, "create"),
            Self::Tag { delete, .. } => push_if(&mut flags, *delete, "delete"),
            Self::Worktree { command } => match command
            {
                Some(WorktreeCommand::Add { branch, .. }) =>
                    push_if(&mut flags, branch.is_some(), "branch"),
                Some(WorktreeCommand::List) | None =>
                {}
                Some(WorktreeCommand::Move { force, .. }) =>
                    push_if(&mut flags, *force > 0, "force"),
                Some(WorktreeCommand::Remove {
                    force,
                    delete,
                    force_delete,
                    ..
                }) =>
                {
                    push_if(&mut flags, *force > 0, "force");
                    push_if(&mut flags, *delete, "delete");
                    push_if(&mut flags, *force_delete, "force_delete");
                }
            },
            Self::Foreach { quiet, .. } => push_if(&mut flags, *quiet, "quiet"),
            Self::Clone { .. }
            | Self::Init { .. }
            | Self::Mv { .. }
            | Self::Status
            | Self::Merge { .. }
            | Self::Rebase { .. }
            | Self::Fetch { .. }
            | Self::Pull { .. }
            | Self::Push { .. } =>
            {}
        }

        flags
    }

    pub fn run(self, context: &CliContext) -> Result<()>
    {
        let working_dir = &context.working_dir;

        match self
        {
            Command::Clone { repository, directory } =>
                clone::clone(working_dir, &repository, directory.as_deref()),

            Command::Init { directory } =>
                init::init(working_dir, directory.as_deref()),

            Command::Add { paths, all, force, chmod } =>
                add::add(working_dir, &paths, all, force, chmod),

            Command::Mv { sources, destination } =>
                mv::mv(working_dir, &sources, &destination),

            Command::Restore { paths, staged, worktree } =>
                restore::restore(working_dir, &paths, worktree, staged),

            Command::Rm { paths, recursive, force, dry_run, cached } =>
                rm::rm(working_dir, &paths, recursive, force, dry_run, cached),

            Command::Status =>
                status::status(&context.display_name, working_dir),

            Command::Branch { delete, force_delete, force, branch_name } =>
                match branch_name
                {
                    Some(branch_name) if delete || force_delete =>
                        branch::delete(
                            working_dir,
                            &branch_name,
                            force || force_delete
                        ),
                    Some(branch_name) =>
                        branch::branch(working_dir, &branch_name),
                    None => branch::branches(working_dir)
                },

            Command::Commit { message } =>
                commit::commit(working_dir, &message),

            Command::Merge { commit_ish } =>
                merge::merge(working_dir, &commit_ish),

            Command::Rebase { upstream } =>
                rebase::rebase(working_dir, &upstream),

            Command::Reset { soft, mixed, hard, merge, keep, commit } =>
                reset::reset(
                    working_dir,
                    ResetMode::from_arg(soft, mixed, hard, merge, keep),
                    commit.as_deref()
                ),

            Command::Switch { create, branch_name } => match create
            {
                true => switch::create(working_dir, &branch_name),
                false => switch::switch(working_dir, &branch_name)
            },

            Command::Tag { delete, tag_name } => match (tag_name, delete)
            {
                (Some(tag_name), true) => tag::delete(working_dir, &tag_name),
                (Some(tag_name), false) => tag::create(working_dir, &tag_name),
                (None, false) => tag::tag(working_dir),
                _ => unreachable!()
            },

            Command::Fetch { repository, refspecs } =>
                fetch::fetch(working_dir, repository.as_deref(), &refspecs),

            Command::Pull { repository, refspecs } =>
                pull::pull(working_dir, repository.as_deref(), &refspecs),

            Command::Push { repository, refspecs } =>
                push::push(working_dir, repository.as_deref(), &refspecs),

            Command::Worktree { command } => match command
                .unwrap_or(WorktreeCommand::List)
            {
                WorktreeCommand::Add { branch, path, commit_ish } =>
                    worktree::add(
                        working_dir,
                        &path,
                        branch.as_deref(),
                        commit_ish.as_deref()
                    ),
                WorktreeCommand::List => worktree::list(working_dir),
                WorktreeCommand::Move { force, path, new_path } =>
                    worktree::move_worktree(
                        working_dir,
                        &path,
                        &new_path,
                        force
                    ),
                WorktreeCommand::Remove {
                    force,
                    delete,
                    force_delete,
                    path
                } => worktree::remove(
                    working_dir,
                    &path,
                    force,
                    delete,
                    force_delete
                )
            },

            Command::Foreach { quiet, command } =>
                foreach::foreach(working_dir, quiet, &command),
        }
    }
}

fn push_if(flags: &mut Vec<&'static str>, condition: bool, flag: &'static str)
{
    if condition
    {
        flags.push(flag);
    }
}
