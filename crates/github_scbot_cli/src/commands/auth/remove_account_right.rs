use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// remove right from account.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-account-right")]
pub(crate) struct AuthRemoveAccountRightCommand {
    /// account username.
    #[argh(positional)]
    username: String,
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAccountRightCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();
        let mut repo_db = ctx.db_adapter.repositories();

        let _repo = CliDbExt::get_existing_repository(&mut *repo_db, owner, name).await?;
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;

        exr_db.delete(owner, name, &self.username).await?;
        println!(
            "Right removed to repository '{}' for account '{}'.",
            self.repository_path, self.username
        );

        Ok(())
    }
}
