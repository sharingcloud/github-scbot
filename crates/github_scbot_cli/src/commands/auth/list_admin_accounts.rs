use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list admin accounts.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-admin-accounts")]
pub(crate) struct AuthListAdminAccountsCommand {}

#[async_trait(?Send)]
impl Command for AuthListAdminAccountsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let accounts = ctx.db_adapter.accounts().list_admins().await?;
        if accounts.is_empty() {
            println!("No admin account found.");
        } else {
            println!("Admin accounts:");
            for account in accounts {
                println!("- {}", account.username());
            }
        }

        Ok(())
    }
}
