use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list rights for account.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-account-rights")]
pub(crate) struct AuthListAccountRightsCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthListAccountRightsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let account = ctx
            .db_adapter
            .external_account()
            .get_from_username(&self.username)
            .await?;
        let rights = ctx
            .db_adapter
            .external_account_right()
            .list_rights(&account.username)
            .await?;
        if rights.is_empty() {
            println!("No right found from account '{}'.", self.username);
        } else {
            println!("Rights from account '{}':", self.username);
            for right in rights {
                if let Ok(repo) = right.get_repository(ctx.db_adapter.repository()).await {
                    println!("- {}", repo.get_path());
                }
            }
        }

        Ok(())
    }
}
