use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

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
        let mut exa_db = ctx.db_adapter.external_accounts();
        let exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        println!("{}", exa.generate_access_token()?);

        Ok(())
    }
}
