use async_trait::async_trait;
use clap::{Parser, Subcommand};

use self::{add::AddCommand, list::ListCommand, remove::RemoveCommand};
use crate::{
    commands::{Command, CommandContext},
    Result,
};

mod add;
mod list;
mod remove;

/// Commands around external accounts
#[derive(Parser)]
pub(crate) struct PullRequestRuleCommand {
    #[clap(subcommand)]
    inner: PullRequestRuleSubCommand,
}

#[async_trait]
impl Command for PullRequestRuleCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum PullRequestRuleSubCommand {
    Add(AddCommand),
    List(ListCommand),
    Remove(RemoveCommand),
}

#[async_trait]
impl Command for PullRequestRuleSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Add(sub) => sub.execute(ctx).await,
            Self::Remove(sub) => sub.execute(ctx).await,
            Self::List(sub) => sub.execute(ctx).await,
        }
    }
}
