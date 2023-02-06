//! Auth commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

mod add_admin_rights;
mod add_external_account_right;
mod create_external_account;
mod create_external_token;
mod list_account_rights;
mod list_admin_accounts;
mod list_external_accounts;
mod remove_account_right;
mod remove_account_rights;
mod remove_admin_rights;
mod remove_external_account;

use self::{
    add_admin_rights::AuthAddAdminRightsCommand,
    add_external_account_right::AuthAddExternalAccountRightCommand,
    create_external_account::AuthCreateExternalAccountCommand,
    create_external_token::AuthCreateExternalTokenCommand,
    list_account_rights::AuthListAccountRightsCommand,
    list_admin_accounts::AuthListAdminAccountsCommand,
    list_external_accounts::AuthListExternalAccountsCommand,
    remove_account_right::AuthRemoveAccountRightCommand,
    remove_account_rights::AuthRemoveAccountRightsCommand,
    remove_admin_rights::AuthRemoveAdminRightsCommand,
    remove_external_account::AuthRemoveExternalAccountCommand,
};

/// Authentication related commands
#[derive(Parser)]
pub(crate) struct AuthCommand {
    #[clap(subcommand)]
    inner: AuthSubCommand,
}

#[async_trait(?Send)]
impl Command for AuthCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum AuthSubCommand {
    CreateExternalAccount(AuthCreateExternalAccountCommand),
    CreateExternalToken(AuthCreateExternalTokenCommand),
    RemoveExternalAccount(AuthRemoveExternalAccountCommand),
    ListExternalAccounts(AuthListExternalAccountsCommand),
    AddExternalAccountRight(AuthAddExternalAccountRightCommand),
    RemoveAccountRight(AuthRemoveAccountRightCommand),
    RemoveAccountRights(AuthRemoveAccountRightsCommand),
    ListAccountRights(AuthListAccountRightsCommand),
    AddAdminRights(AuthAddAdminRightsCommand),
    RemoveAdminRights(AuthRemoveAdminRightsCommand),
    ListAdminAccounts(AuthListAdminAccountsCommand),
}

#[async_trait(?Send)]
impl Command for AuthSubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::CreateExternalAccount(sub) => sub.run(ctx).await,
            Self::CreateExternalToken(sub) => sub.run(ctx).await,
            Self::RemoveExternalAccount(sub) => sub.run(ctx).await,
            Self::ListExternalAccounts(sub) => sub.run(ctx).await,
            Self::AddExternalAccountRight(sub) => sub.run(ctx).await,
            Self::RemoveAccountRight(sub) => sub.run(ctx).await,
            Self::RemoveAccountRights(sub) => sub.run(ctx).await,
            Self::ListAccountRights(sub) => sub.run(ctx).await,
            Self::AddAdminRights(sub) => sub.run(ctx).await,
            Self::RemoveAdminRights(sub) => sub.run(ctx).await,
            Self::ListAdminAccounts(sub) => sub.run(ctx).await,
        }
    }
}
