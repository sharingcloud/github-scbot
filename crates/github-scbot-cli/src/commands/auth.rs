//! Auth commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

mod add_admin_rights;
mod add_external_account;
mod add_external_account_right;
mod generate_external_token;
mod list_admin_accounts;
mod list_external_account_rights;
mod list_external_accounts;
mod remove_admin_rights;
mod remove_all_external_account_rights;
mod remove_external_account;
mod remove_external_account_right;

use self::{
    add_admin_rights::AuthAddAdminRightsCommand,
    add_external_account::AuthAddExternalAccountCommand,
    add_external_account_right::AuthAddExternalAccountRightCommand,
    generate_external_token::AuthGenerateExternalTokenCommand,
    list_admin_accounts::AuthListAdminAccountsCommand,
    list_external_account_rights::AuthListExternalAccountRightsCommand,
    list_external_accounts::AuthListExternalAccountsCommand,
    remove_admin_rights::AuthRemoveAdminRightsCommand,
    remove_all_external_account_rights::AuthRemoveAllExternalAccountRightsCommand,
    remove_external_account::AuthRemoveExternalAccountCommand,
    remove_external_account_right::AuthRemoveExternalAccountRightCommand,
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
    AddAdminRights(AuthAddAdminRightsCommand),
    AddExternalAccount(AuthAddExternalAccountCommand),
    AddExternalAccountRight(AuthAddExternalAccountRightCommand),
    GenerateExternalToken(AuthGenerateExternalTokenCommand),
    ListAdminAccounts(AuthListAdminAccountsCommand),
    ListExternalAccounts(AuthListExternalAccountsCommand),
    ListExternalAccountRights(AuthListExternalAccountRightsCommand),
    RemoveExternalAccount(AuthRemoveExternalAccountCommand),
    RemoveExternalAccountRight(AuthRemoveExternalAccountRightCommand),
    RemoveAllExternalAccountRights(AuthRemoveAllExternalAccountRightsCommand),
    RemoveAdminRights(AuthRemoveAdminRightsCommand),
}

#[async_trait(?Send)]
impl Command for AuthSubCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::AddAdminRights(sub) => sub.run(&mut ctx).await,
            Self::AddExternalAccount(sub) => sub.run(ctx).await,
            Self::AddExternalAccountRight(sub) => sub.run(ctx).await,
            Self::GenerateExternalToken(sub) => sub.run(ctx).await,
            Self::RemoveExternalAccount(sub) => sub.run(ctx).await,
            Self::ListAdminAccounts(sub) => sub.run(ctx).await,
            Self::ListExternalAccounts(sub) => sub.run(ctx).await,
            Self::ListExternalAccountRights(sub) => sub.run(ctx).await,
            Self::RemoveExternalAccountRight(sub) => sub.run(ctx).await,
            Self::RemoveAllExternalAccountRights(sub) => sub.run(ctx).await,
            Self::RemoveAdminRights(sub) => sub.run(ctx).await,
        }
    }
}
