use std::io::Write;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_types::repository::RepositoryPath;
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Remove right from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightCommand {
    /// Account username
    username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAccountRightCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();
        let mut repo_db = ctx.db_adapter.repositories();

        let _repo = CliDbExt::get_existing_repository(&mut *repo_db, owner, name).await?;
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;

        exr_db
            .delete(owner, name, &self.username)
            .await
            .context(DatabaseSnafu)?;
        writeln!(
            ctx.writer,
            "Right removed to repository '{}' for account '{}'.",
            self.repository_path, self.username
        )
        .context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{
        use_temporary_db, DbService, DbServiceImplPool, ExternalAccount, ExternalAccountRight,
        Repository,
    };
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_remove_account_right",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let repo = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;
                db_adapter
                    .external_accounts()
                    .create(ExternalAccount::builder().username("me").build()?)
                    .await?;
                db_adapter
                    .external_account_rights()
                    .create(
                        ExternalAccountRight::builder()
                            .with_repository(&repo)
                            .username("me")
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "remove-account-right", "me", "owner/name"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Right removed to repository 'owner/name' for account 'me'.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter
                        .external_account_rights()
                        .get("owner", "name", "me")
                        .await?
                        .is_none(),
                    "external account 'me' should not have right on 'owner/name'"
                );

                Ok(())
            },
        )
        .await;
    }
}
