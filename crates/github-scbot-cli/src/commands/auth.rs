//! Auth commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use github_scbot_domain::use_cases::auth::{
    AddAccountRightUseCase, AddAdminRightUseCase, CreateExternalAccountUseCase,
    CreateExternalTokenUseCase, ListAccountRightsUseCase, ListAdminAccountsUseCase,
    ListExternalAccountsUseCase, RemoveAccountRightUseCase, RemoveAdminRightUseCase,
    RemoveAllAccountRightsUseCase, RemoveExternalAccountUseCase,
};

use super::{Command, CommandContext};

mod add_account_right;
mod add_admin_rights;
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
    add_account_right::AuthAddAccountRightCommand, add_admin_rights::AuthAddAdminRightsCommand,
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
    AddAccountRight(AuthAddAccountRightCommand),
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
            Self::CreateExternalAccount(sub) => {
                let use_case = CreateExternalAccountUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::CreateExternalToken(sub) => {
                let use_case = CreateExternalTokenUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::RemoveExternalAccount(sub) => {
                let use_case = RemoveExternalAccountUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::ListExternalAccounts(sub) => {
                let use_case = ListExternalAccountsUseCase {
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::AddAccountRight(sub) => {
                let use_case = AddAccountRightUseCase {
                    username: sub.username.clone(),
                    repository_path: sub.repository_path.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::RemoveAccountRight(sub) => {
                let use_case = RemoveAccountRightUseCase {
                    username: sub.username.clone(),
                    repository_path: sub.repository_path.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::RemoveAccountRights(sub) => {
                let use_case = RemoveAllAccountRightsUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::ListAccountRights(sub) => {
                let use_case = ListAccountRightsUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::AddAdminRights(sub) => {
                let use_case = AddAdminRightUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::RemoveAdminRights(sub) => {
                let use_case = RemoveAdminRightUseCase {
                    username: sub.username.clone(),
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
            Self::ListAdminAccounts(sub) => {
                let use_case = ListAdminAccountsUseCase {
                    db_service: ctx.db_adapter.as_ref(),
                };

                sub.run(ctx.writer, &use_case).await
            }
        }
    }
}
