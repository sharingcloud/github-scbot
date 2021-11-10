use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// remove external account.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-external-account")]
pub(crate) struct AuthRemoveExternalAccountCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveExternalAccountCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let account = ctx
            .db_adapter
            .external_account()
            .get_from_username(&self.username)
            .await?;
        ctx.db_adapter.external_account().remove(account).await?;

        println!("External account '{}' removed.", self.username);

        Ok(())
    }
}
