use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::AccountModel;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// remove admin rights from account.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-admin-rights")]
pub(crate) struct AuthRemoveAdminRightsCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAdminRightsCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        AccountModel::builder(&self.username)
            .admin(false)
            .create_or_update(ctx.db_adapter.account())
            .await?;

        println!(
            "Account '{}' added/edited without admin rights.",
            self.username
        );

        Ok(())
    }
}
