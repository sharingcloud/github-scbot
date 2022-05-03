use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Account;
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
        let mut acc_db = ctx.db_adapter.accounts();
        match acc_db.get(&self.username).await? {
            Some(_) => acc_db.set_is_admin(&self.username, true).await?,
            None => {
                acc_db
                    .create(
                        Account::builder()
                            .username(self.username.clone())
                            .is_admin(true)
                            .build()?,
                    )
                    .await?
            }
        };

        println!(
            "Account '{}' added/edited with admin rights.",
            self.username
        );

        Ok(())
    }
}
