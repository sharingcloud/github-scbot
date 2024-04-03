use async_trait::async_trait;
use clap::{Parser, Subcommand};

use self::{add::AuthAdminAddCommand, list::AuthAdminListCommand, remove::AuthAdminRemoveCommand};
use crate::{
    commands::{Command, CommandContext},
    Result,
};

mod add;
mod list;
mod remove;

/// Commands around admins
#[derive(Parser)]
pub(crate) struct AuthAdminCommand {
    #[clap(subcommand)]
    inner: AuthAdminSubCommand,
}

#[async_trait]
impl Command for AuthAdminCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum AuthAdminSubCommand {
    Add(AuthAdminAddCommand),
    List(AuthAdminListCommand),
    Remove(AuthAdminRemoveCommand),
}

#[async_trait]
impl Command for AuthAdminSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Add(sub) => sub.run(ctx).await,
            Self::List(sub) => sub.run(ctx).await,
            Self::Remove(sub) => sub.run(ctx).await,
        }
    }
}
