//! Auth commands.

use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

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

/// authentication related commands.
#[derive(FromArgs)]
#[argh(subcommand, name = "auth")]
pub(crate) struct AuthCommand {
    #[argh(subcommand)]
    inner: AuthSubCommand,
}

#[async_trait(?Send)]
impl Command for AuthCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(FromArgs)]
#[argh(subcommand)]
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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::CreateExternalAccount(sub) => sub.execute(ctx).await,
            Self::CreateExternalToken(sub) => sub.execute(ctx).await,
            Self::RemoveExternalAccount(sub) => sub.execute(ctx).await,
            Self::ListExternalAccounts(sub) => sub.execute(ctx).await,
            Self::AddAccountRight(sub) => sub.execute(ctx).await,
            Self::RemoveAccountRight(sub) => sub.execute(ctx).await,
            Self::RemoveAccountRights(sub) => sub.execute(ctx).await,
            Self::ListAccountRights(sub) => sub.execute(ctx).await,
            Self::AddAdminRights(sub) => sub.execute(ctx).await,
            Self::RemoveAdminRights(sub) => sub.execute(ctx).await,
            Self::ListAdminAccounts(sub) => sub.execute(ctx).await,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use github_scbot_database::models::DummyDatabaseAdapter;

//     use super::*;

//     fn arrange() -> DummyDatabaseAdapter {
//         DummyDatabaseAdapter::new()
//     }

//     #[actix_rt::test]
//     async fn test_create_external_account() -> Result<()> {
//         create_external_account(&arrange(), "test").await
//     }

//     #[actix_rt::test]
//     async fn test_list_external_accounts() -> Result<()> {
//         list_external_accounts(&arrange()).await
//     }

//     #[actix_rt::test]
//     async fn test_remove_external_accounts() -> Result<()> {
//         remove_external_account(&arrange(), "test").await
//     }

//     #[actix_rt::test]
//     async fn test_create_external_token() -> Result<()> {
//         let mut adapter = arrange();
//         adapter
//             .external_account_adapter
//             .get_from_username_response
//             .set_response(Ok(ExternalAccountModel::builder("test")
//                 .generate_keys()
//                 .build()));

//         create_external_token(&adapter, "test").await
//     }

//     #[actix_rt::test]
//     async fn test_add_account_right() -> Result<()> {
//         add_account_right(&arrange(), "test", "repo/path").await
//     }

//     #[actix_rt::test]
//     async fn test_remove_account_right() -> Result<()> {
//         remove_account_right(&arrange(), "test", "repo/path").await
//     }

//     #[actix_rt::test]
//     async fn test_remove_account_rights() -> Result<()> {
//         remove_account_rights(&arrange(), "test").await
//     }

//     #[actix_rt::test]
//     async fn test_list_account_rights() -> Result<()> {
//         list_account_rights(&arrange(), "test").await
//     }
// }
