use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list external accounts.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-external-accounts")]
pub(crate) struct AuthListExternalAccountsCommand {}

#[async_trait(?Send)]
impl Command for AuthListExternalAccountsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let accounts = ctx.db_adapter.external_accounts().list().await?;
        if accounts.is_empty() {
            println!("No external account found.");
        } else {
            println!("External accounts:");
            for account in accounts {
                println!("- {}", account.username());
            }
        }

        Ok(())
    }
}
