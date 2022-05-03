use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// remove all rights from account.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-account-rights")]
pub(crate) struct AuthRemoveAccountRightsCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAccountRightsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;

        exr_db.delete_all(&self.username).await?;
        println!("All rights removed from account '{}'.", self.username);

        Ok(())
    }
}
