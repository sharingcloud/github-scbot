use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// list external accounts.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-external-accounts")]
pub(crate) struct AuthListExternalAccountsCommand {}

#[async_trait(?Send)]
impl Command for AuthListExternalAccountsCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let accounts = ctx.db_adapter.external_account().list().await?;
        if accounts.is_empty() {
            println!("No external account found.");
        } else {
            println!("External accounts:");
            for account in accounts {
                println!("- {}", account.username);
            }
        }

        Ok(())
    }
}
