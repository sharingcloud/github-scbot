use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};
use crate::Result;

mod apply_pull_request_rules;
mod list;
mod set_merge_strategy;
mod show;
mod sync;

use self::{
    apply_pull_request_rules::PullRequestApplyPullRequestRulesCommand,
    list::PullRequestListCommand, set_merge_strategy::PullRequestSetMergeStrategyCommand,
    show::PullRequestShowCommand, sync::PullRequestSyncCommand,
};

/// Manage pull requests
#[derive(Parser)]
pub(crate) struct PullRequestCommand {
    #[clap(subcommand)]
    inner: PullRequestSubCommand,
}

#[async_trait]
impl Command for PullRequestCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
pub(crate) enum PullRequestSubCommand {
    Show(PullRequestShowCommand),
    Sync(PullRequestSyncCommand),
    SetMergeStrategy(PullRequestSetMergeStrategyCommand),
    ApplyPullRequestRules(PullRequestApplyPullRequestRulesCommand),
    List(PullRequestListCommand),
}

#[async_trait]
impl Command for PullRequestSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::List(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::Sync(sub) => sub.execute(ctx).await,
            Self::ApplyPullRequestRules(sub) => sub.execute(ctx).await,
            Self::SetMergeStrategy(sub) => sub.execute(ctx).await,
        }
    }
}
