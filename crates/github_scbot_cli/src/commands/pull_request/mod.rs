use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

mod list;
mod set_merge_strategy;
mod show;
mod sync;

use self::{
    list::PullRequestListCommand, set_merge_strategy::PullRequestSetMergeStrategyCommand,
    show::PullRequestShowCommand, sync::PullRequestSyncCommand,
};

/// Manage pull requests
#[derive(Parser)]
pub(crate) struct PullRequestCommand {
    #[clap(subcommand)]
    inner: PullRequestSubCommand,
}

#[async_trait(?Send)]
impl Command for PullRequestCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
pub(crate) enum PullRequestSubCommand {
    Show(PullRequestShowCommand),
    Sync(PullRequestSyncCommand),
    SetMergeStrategy(PullRequestSetMergeStrategyCommand),
    List(PullRequestListCommand),
}

#[async_trait(?Send)]
impl Command for PullRequestSubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::List(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::Sync(sub) => sub.execute(ctx).await,
            Self::SetMergeStrategy(sub) => sub.execute(ctx).await,
        }
    }
}
