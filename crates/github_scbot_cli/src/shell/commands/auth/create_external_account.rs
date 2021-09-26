use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::ExternalAccountModel;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

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
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        ExternalAccountModel::builder(&self.username)
            .generate_keys()
            .create_or_update(ctx.db_adapter.external_account())
            .await?;
        println!("External account '{}' created.", self.username);

        Ok(())
    }
}
