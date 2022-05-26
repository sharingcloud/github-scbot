use std::io::Write;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// remove all rights from account.
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightsCommand {
    /// account username.
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAccountRightsCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;

        exr_db
            .delete_all(&self.username)
            .await
            .context(DatabaseSnafu)?;
        writeln!(
            ctx.writer,
            "All rights removed from account '{}'.",
            self.username
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
            "test_command_remove_account_rights",
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
                    &["auth", "remove-account-rights", "me"],
                )
                .await?;

                assert_eq!(output, "All rights removed from account 'me'.\n");

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
