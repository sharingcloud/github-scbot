use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// add right to account.
#[derive(FromArgs)]
#[argh(subcommand, name = "add-account-right")]
pub(crate) struct AuthAddAccountRightCommand {
    /// account username.
    #[argh(positional)]
    username: String,
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for AuthAddAccountRightCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
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
            .add_right(&account.username, &repo)
            .await?;
        println!(
            "Right added to repository '{}' for account '{}'.",
            self.repository_path, self.username
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AuthAddAccountRightCommand;
    use crate::{commands::Command, tests::create_test_context};

    #[actix_rt::test]
    async fn test_command() {
        let context = create_test_context();

        let command = AuthAddAccountRightCommand {
            username: "me".into(),
            repository_path: "me/repo".into(),
        };

        command.execute(context).await.unwrap();
    }
}
