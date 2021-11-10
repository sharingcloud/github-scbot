use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use super::{Command, CommandContext};

mod list;
mod set_merge_strategy;
mod show;
mod sync;

use self::{
    list::PullRequestListCommand, set_merge_strategy::PullRequestSetMergeStrategyCommand,
    show::PullRequestShowCommand, sync::PullRequestSyncCommand,
};

/// manage pull requests.
#[derive(FromArgs)]
#[argh(subcommand, name = "pull-requests")]
pub(crate) struct PullRequestCommand {
    #[argh(subcommand)]
    inner: PullRequestSubCommand,
}

#[async_trait(?Send)]
impl Command for PullRequestCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub(crate) enum PullRequestSubCommand {
    Show(PullRequestShowCommand),
    Sync(PullRequestSyncCommand),
    SetMergeStrategy(PullRequestSetMergeStrategyCommand),
    List(PullRequestListCommand),
}

#[async_trait(?Send)]
impl Command for PullRequestSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::List(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::Sync(sub) => sub.execute(ctx).await,
            Self::SetMergeStrategy(sub) => sub.execute(ctx).await,
        }
    }
}
