use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

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
        let mut repo_db = ctx.db_adapter.repositories();
        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();

        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        let rights = exr_db.list(&self.username).await?;

        if rights.is_empty() {
            println!("No right found from account '{}'.", self.username);
        } else {
            println!("Rights from account '{}':", self.username);
            for right in rights {
                let repo = repo_db.get_from_id(right.repository_id()).await?.unwrap();
                println!("- {}/{}", repo.owner(), repo.name());
            }
        }

        Ok(())
    }
}
