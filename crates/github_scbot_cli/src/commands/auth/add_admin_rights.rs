use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::AccountModel;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
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
