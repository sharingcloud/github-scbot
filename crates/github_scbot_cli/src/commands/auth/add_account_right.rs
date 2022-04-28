use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::ExternalAccountRight;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// add right to account.
#[derive(FromArgs)]
#[argh(subcommand, name = "add-account-right")]
pub(crate) struct AuthAddAccountRightCommand {
    /// account username.
    #[argh(positional)]
    username: String,
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for AuthAddAccountRightCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut repo_db = ctx.db_adapter.repositories();
        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();

        let repo = CliDbExt::get_existing_repository(&mut *repo_db, owner, name).await?;
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        exr_db.delete(owner, name, &self.username).await?;
        exr_db
            .create(
                ExternalAccountRight::builder()
                    .repository_id(repo.id())
                    .username(self.username.clone())
                    .build()?,
            )
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
    use std::str::FromStr;

    use github_scbot_conf::Config;
    use github_scbot_database2::MockDbService;
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;
    use github_scbot_types::repository::RepositoryPath;

    use super::AuthAddAccountRightCommand;
    use crate::commands::{Command, CommandContext};

    #[tokio::test]
    async fn test_command() {
        let api_adapter = Box::new(MockApiService::new());
        let db_adapter = Box::new(MockDbService::new());
        let redis_adapter = Box::new(MockRedisService::new());

        let context = CommandContext {
            config: Config::from_env(),
            api_adapter,
            db_adapter,
            redis_adapter,
            no_input: true,
        };

        let command = AuthAddAccountRightCommand {
            username: "me".into(),
            repository_path: RepositoryPath::from_str("repo/me").unwrap(),
        };

        command.execute(context).await.unwrap();
    }
}
