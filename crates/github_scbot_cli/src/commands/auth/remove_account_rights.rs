use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

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
        let account = ctx
            .db_adapter
            .external_account()
            .get_from_username(&self.username)
            .await?;

        ctx.db_adapter
            .external_account_right()
            .remove_rights(&account.username)
            .await?;
        println!("All rights removed from account '{}'.", self.username);

        Ok(())
    }
}
