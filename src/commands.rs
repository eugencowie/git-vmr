mod add;
mod branch;
mod clone;
mod commit;
mod fetch;
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

use crate::git::ResetMode;
use anyhow::Result;
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum WorktreeCommand
{
    /// Create a worktree at [path] and checkout [commit-ish] into it
    Add
    {
        #[arg(value_name = "path")]
        path: PathBuf,

        #[arg(value_name = "commit-ish")]
        commit_ish: Option<String>
    }
}

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
        /// Files to add content from
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Move or rename a file, a directory, or a symlink
    Mv
    {
        /// File to move
        #[arg(value_name = "source")]
        source: PathBuf,

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
        /// different between <commit> and HEAD, but keep those which are
        /// different between the index and working tree (i.e. which have
        /// changes which have not been added)
        #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "keep"])]
        merge: bool,

        /// Resets index entries and updates files in the working tree that are
        /// different between <commit> and HEAD
        #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "merge"])]
        keep: bool,

        /// Set the current branch head (HEAD) to point at <commit>
        #[arg(value_name = "commit")]
        commit: Option<String>
    },

    /// Switch branches
    Switch
    {
        /// Create a new branch named [branch] before switching to the branch
        #[arg(short = 'c', long)]
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
        command: WorktreeCommand
    }
}

impl Command
{
    pub fn run(self, bin_name: &str, working_dir: &Path) -> Result<()>
    {
        match self
        {
            Command::Clone { repository, directory } =>
                clone::clone(working_dir, &repository, directory.as_deref()),

            Command::Init { directory } =>
                init::init(working_dir, directory.as_deref()),

            Command::Add { paths } => add::add(working_dir, &paths),

            Command::Mv { source, destination } =>
                mv::mv(working_dir, &source, &destination),

            Command::Restore { paths, staged, worktree } =>
                restore::restore(working_dir, &paths, worktree, staged),

            Command::Rm { paths, recursive } =>
                rm::rm(working_dir, &paths, recursive),

            Command::Status => status::status(bin_name, working_dir),

            Command::Branch { delete, force_delete, force, branch_name } =>
                match (branch_name, delete, force_delete, force)
                {
                    (Some(branch_name), true, false, false) =>
                        branch::delete(working_dir, &branch_name),
                    (Some(branch_name), false, true, false) =>
                        branch::force_delete(working_dir, &branch_name),
                    (Some(branch_name), true, false, true) =>
                        branch::force_delete(working_dir, &branch_name),
                    (Some(branch_name), false, false, false) =>
                        branch::branch(working_dir, &branch_name),
                    (None, false, false, false) =>
                        branch::branches(working_dir),
                    _ => unreachable!()
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
            {
                WorktreeCommand::Add { path, commit_ish } =>
                    worktree::add(working_dir, &path, commit_ish.as_deref()),
            }
        }
    }
}
