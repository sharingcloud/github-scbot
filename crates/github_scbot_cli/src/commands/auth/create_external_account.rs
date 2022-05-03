use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::ExternalAccount;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// create external account.
#[derive(FromArgs)]
#[argh(subcommand, name = "create-external-account")]
pub(crate) struct AuthCreateExternalAccountCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthCreateExternalAccountCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();

        exa_db
            .create(
                ExternalAccount::builder()
                    .username(self.username.clone())
                    .generate_keys()
                    .build()?,
            )
            .await?;

        println!("External account '{}' created.", self.username);

        Ok(())
    }
}
