use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// remove right from account.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-account-right")]
pub(crate) struct AuthRemoveAccountRightCommand {
    /// account username.
    #[argh(positional)]
    username: String,
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAccountRightCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        let account = ctx
            .db_adapter
            .external_account()
            .get_from_username(&self.username)
            .await?;

        ctx.db_adapter
            .external_account_right()
            .remove_right(&account.username, &repo)
            .await?;
        println!(
            "Right removed to repository '{}' for account '{}'.",
            self.repository_path, self.username
        );

        Ok(())
    }
}
