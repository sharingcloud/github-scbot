use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// create external token.
#[derive(FromArgs)]
#[argh(subcommand, name = "create-external-token")]
pub(crate) struct AuthCreateExternalTokenCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthCreateExternalTokenCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let account = ctx
            .db_adapter
            .external_account()
            .get_from_username(&self.username)
            .await?;
        println!("{}", account.generate_access_token()?);

        Ok(())
    }
}
