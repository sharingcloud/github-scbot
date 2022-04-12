use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

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
        let mut exa_db = ctx.db_adapter.external_accounts();
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        exa_db.delete(&self.username).await?;

        println!("External account '{}' removed.", self.username);

        Ok(())
    }
}
