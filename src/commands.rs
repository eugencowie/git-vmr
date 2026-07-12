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
use crate::git::{
    AddArgs, BranchArgs, CloneArgs, CommitArgs, FetchArgs, MergeArgs, PullArgs,
    PushArgs, RebaseArgs, ResetArgs, RestoreArgs, RmArgs, SwitchArgs, TagArgs
};
use crate::render::Rendered;
use crate::workspace::{MvArgs, Workspace, WorktreeCommand};
use anyhow::Result;
use clap::Subcommand;
pub use foreach::ForeachArgs;
use init::InitArgs;

#[derive(Subcommand)]
pub enum Command
{
    // Clone and init run before a VMR exists
    Clone(CloneArgs),
    Init(InitArgs),

    // Every other command is a workspace command
    #[command(flatten)]
    Workspace(WorkspaceCommand)
}

/// A command that runs inside an opened workspace: the VMR is discovered and
/// its child repos snapshotted before the command sees anything.
#[derive(Subcommand)]
pub enum WorkspaceCommand
{
    Add(AddArgs),
    Mv(MvArgs),
    Restore(RestoreArgs),
    Rm(RmArgs),

    /// Show the working tree status
    Status,

    Branch(BranchArgs),
    Commit(CommitArgs),
    Merge(MergeArgs),
    Rebase(RebaseArgs),
    Reset(ResetArgs),
    Switch(SwitchArgs),
    Tag(TagArgs),
    Fetch(FetchArgs),
    Pull(PullArgs),
    Push(PushArgs),

    /// Manage multiple working trees
    Worktree
    {
        #[command(subcommand)]
        command: Option<WorktreeCommand>
    },

    Foreach(ForeachArgs)
}

impl Command
{
    pub fn run(self, context: &CliContext) -> Result<Rendered>
    {
        let working_dir = &context.working_dir;
        let git = &context.git;

        match self
        {
            Command::Clone(args) => clone::clone(git, working_dir, &args),

            Command::Init(args) => init::init(working_dir, &args),

            Command::Workspace(command) =>
            {
                let workspace = Workspace::find(git, working_dir)?;
                command.run(&workspace, context)
            }
        }
    }
}

impl WorkspaceCommand
{
    fn run(
        self,
        workspace: &Workspace,
        context: &CliContext
    ) -> Result<Rendered>
    {
        let working_dir = &context.working_dir;

        match self
        {
            WorkspaceCommand::Add(args) =>
                add::add(workspace, working_dir, &args),

            WorkspaceCommand::Mv(args) => mv::mv(workspace, working_dir, &args),

            WorkspaceCommand::Restore(args) =>
                restore::restore(workspace, working_dir, &args),

            WorkspaceCommand::Rm(args) => rm::rm(workspace, working_dir, &args),

            WorkspaceCommand::Status =>
                status::status(workspace, &context.display_name, working_dir),

            WorkspaceCommand::Branch(args) => branch::branch(workspace, &args),

            WorkspaceCommand::Commit(args) => commit::commit(workspace, &args),

            WorkspaceCommand::Merge(args) => merge::merge(workspace, &args),

            WorkspaceCommand::Rebase(args) => rebase::rebase(workspace, &args),

            WorkspaceCommand::Reset(args) => reset::reset(workspace, &args),

            WorkspaceCommand::Switch(args) => switch::switch(workspace, &args),

            WorkspaceCommand::Tag(args) => tag::tag(workspace, &args),

            WorkspaceCommand::Fetch(args) => fetch::fetch(workspace, &args),

            WorkspaceCommand::Pull(args) => pull::pull(workspace, &args),

            WorkspaceCommand::Push(args) => push::push(workspace, &args),

            WorkspaceCommand::Worktree { command } =>
                worktree::worktree(workspace, working_dir, command),

            WorkspaceCommand::Foreach(args) =>
                foreach::foreach(workspace, working_dir, &args),
        }
    }
}
