use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::AccountModel;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// add admin rights to account.
#[derive(FromArgs)]
#[argh(subcommand, name = "add-admin-rights")]
pub(crate) struct AuthAddAdminRightsCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthAddAdminRightsCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        AccountModel::builder(&self.username)
            .admin(true)
            .create_or_update(ctx.db_adapter.account())
            .await?;

        println!(
            "Account '{}' added/edited with admin rights.",
            self.username
        );

        Ok(())
    }
}
