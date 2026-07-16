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
use crate::render::Rendered;
use crate::workspace::Workspace;
pub use add::AddArgs;
use anyhow::Result;
pub use branch::BranchArgs;
use clap::Subcommand;
pub use clone::CloneArgs;
pub use commit::CommitArgs;
pub use fetch::FetchArgs;
pub use foreach::ForeachArgs;
use init::InitArgs;
pub use merge::MergeArgs;
pub use mv::MvArgs;
pub use pull::PullArgs;
pub use push::PushArgs;
pub use rebase::RebaseArgs;
pub use reset::ResetArgs;
pub use restore::RestoreArgs;
pub use rm::RmArgs;
pub use switch::SwitchArgs;
pub use tag::TagArgs;
pub use worktree::WorktreeCommand;

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

// A workspace command runs inside an opened workspace: the VMR is discovered
// and its child repos snapshotted before the command sees anything. Not a doc
// comment — clap would surface it as the application's help text.
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
        match self
        {
            Command::Clone(args) => clone::run(context, args),

            Command::Init(args) => init::run(context, args),

            Command::Workspace(command) =>
            {
                let workspace =
                    Workspace::find(&context.git, &context.working_dir)?;
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
        match self
        {
            WorkspaceCommand::Add(args) => add::run(workspace, context, args),

            WorkspaceCommand::Mv(args) => mv::run(workspace, context, args),

            WorkspaceCommand::Restore(args) =>
                restore::run(workspace, context, args),

            WorkspaceCommand::Rm(args) => rm::run(workspace, context, args),

            WorkspaceCommand::Status => status::run(workspace, context),

            WorkspaceCommand::Branch(args) =>
                branch::run(workspace, context, args),

            WorkspaceCommand::Commit(args) =>
                commit::run(workspace, context, args),

            WorkspaceCommand::Merge(args) =>
                merge::run(workspace, context, args),

            WorkspaceCommand::Rebase(args) =>
                rebase::run(workspace, context, args),

            WorkspaceCommand::Reset(args) =>
                reset::run(workspace, context, args),

            WorkspaceCommand::Switch(args) =>
                switch::run(workspace, context, args),

            WorkspaceCommand::Tag(args) => tag::run(workspace, context, args),

            WorkspaceCommand::Fetch(args) =>
                fetch::run(workspace, context, args),

            WorkspaceCommand::Pull(args) => pull::run(workspace, context, args),

            WorkspaceCommand::Push(args) => push::run(workspace, context, args),

            WorkspaceCommand::Worktree { command } =>
                worktree::run(workspace, context, command),

            WorkspaceCommand::Foreach(args) =>
                foreach::run(workspace, context, args),
        }
    }
}
