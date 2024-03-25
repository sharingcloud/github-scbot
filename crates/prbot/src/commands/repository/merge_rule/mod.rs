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
pub(crate) struct MergeRuleCommand {
    #[clap(subcommand)]
    inner: MergeRuleSubCommand,
}

#[async_trait]
impl Command for MergeRuleCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum MergeRuleSubCommand {
    Add(AddCommand),
    Remove(RemoveCommand),
    List(ListCommand),
}

#[async_trait]
impl Command for MergeRuleSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Add(sub) => sub.execute(ctx).await,
            Self::List(sub) => sub.execute(ctx).await,
            Self::Remove(sub) => sub.execute(ctx).await,
        }
    }
}
