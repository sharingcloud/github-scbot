use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// list admin accounts.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-admin-accounts")]
pub(crate) struct AuthListAdminAccountsCommand {}

#[async_trait(?Send)]
impl Command for AuthListAdminAccountsCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let accounts = ctx.db_adapter.account().list_admin_accounts().await?;
        if accounts.is_empty() {
            println!("No admin account found.");
        } else {
            println!("Admin accounts:");
            for account in accounts {
                println!("- {}", account.username);
            }
        }

        Ok(())
    }
}
