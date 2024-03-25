use async_trait::async_trait;
use clap::{Parser, Subcommand};

use self::{
    add::AuthExternalAccountAddCommand, add_right::AuthExternalAccountAddRightCommand,
    generate_token::AuthExternalAccountGenerateTokenCommand, list::AuthExternalAccountListCommand,
    list_rights::AuthExternalAccountListRightsCommand, remove::AuthExternalAccountRemoveCommand,
    remove_all_rights::AuthExternalAccountRemoveAllRightsCommand,
    remove_right::AuthExternalAccountRemoveRightCommand,
};
use crate::{
    commands::{Command, CommandContext},
    Result,
};

mod add;
mod add_right;
mod generate_token;
mod list;
mod list_rights;
mod remove;
mod remove_all_rights;
mod remove_right;

/// Commands around external accounts
#[derive(Parser)]
pub(crate) struct AuthExternalAccountCommand {
    #[clap(subcommand)]
    inner: AuthExternalAccountSubCommand,
}

#[async_trait]
impl Command for AuthExternalAccountCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum AuthExternalAccountSubCommand {
    Add(AuthExternalAccountAddCommand),
    AddRight(AuthExternalAccountAddRightCommand),
    GenerateToken(AuthExternalAccountGenerateTokenCommand),
    List(AuthExternalAccountListCommand),
    ListRights(AuthExternalAccountListRightsCommand),
    Remove(AuthExternalAccountRemoveCommand),
    RemoveRight(AuthExternalAccountRemoveRightCommand),
    RemoveAllRights(AuthExternalAccountRemoveAllRightsCommand),
}

#[async_trait]
impl Command for AuthExternalAccountSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Add(sub) => sub.run(ctx).await,
            Self::AddRight(sub) => sub.run(ctx).await,
            Self::GenerateToken(sub) => sub.run(ctx).await,
            Self::List(sub) => sub.run(ctx).await,
            Self::ListRights(sub) => sub.run(ctx).await,
            Self::Remove(sub) => sub.run(ctx).await,
            Self::RemoveRight(sub) => sub.run(ctx).await,
            Self::RemoveAllRights(sub) => sub.run(ctx).await,
        }
    }
}
