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
    Clone(CloneArgs),
    Init(InitArgs),
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
            // Clone and init run before a VMR exists
            Command::Clone(args) => clone::clone(git, working_dir, &args),

            Command::Init(args) => init::init(working_dir, &args),

            // Every other command runs inside an opened workspace
            command =>
            {
                let workspace = Workspace::find(git, working_dir)?;
                command.run_in_workspace(&workspace, context)
            }
        }
    }

    fn run_in_workspace(
        self,
        workspace: &Workspace,
        context: &CliContext
    ) -> Result<Rendered>
    {
        let working_dir = &context.working_dir;

        match self
        {
            Command::Clone(_) | Command::Init(_) => unreachable!(),

            Command::Add(args) => add::add(workspace, working_dir, &args),

            Command::Mv(args) => mv::mv(workspace, working_dir, &args),

            Command::Restore(args) =>
                restore::restore(workspace, working_dir, &args),

            Command::Rm(args) => rm::rm(workspace, working_dir, &args),

            Command::Status =>
                status::status(workspace, &context.display_name, working_dir),

            Command::Branch(args) => branch::branch(workspace, &args),

            Command::Commit(args) => commit::commit(workspace, &args),

            Command::Merge(args) => merge::merge(workspace, &args),

            Command::Rebase(args) => rebase::rebase(workspace, &args),

            Command::Reset(args) => reset::reset(workspace, &args),

            Command::Switch(args) => switch::switch(workspace, &args),

            Command::Tag(args) => tag::tag(workspace, &args),

            Command::Fetch(args) => fetch::fetch(workspace, &args),

            Command::Pull(args) => pull::pull(workspace, &args),

            Command::Push(args) => push::push(workspace, &args),

            Command::Worktree { command } =>
                worktree::worktree(workspace, working_dir, command),

            Command::Foreach(args) =>
                foreach::foreach(workspace, working_dir, &args),
        }
    }
}
