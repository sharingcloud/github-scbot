use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::ResultExt;

/// Remove external account
#[derive(Parser)]
pub(crate) struct AuthRemoveExternalAccountCommand {
    /// Account username
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveExternalAccountCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();
        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        exa_db.delete(&self.username).await.context(DatabaseSnafu)?;

        writeln!(ctx.writer, "External account '{}' removed.", self.username).context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, ExternalAccount};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_remove_external_account",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                db_adapter
                    .external_accounts()
                    .create(ExternalAccount::builder().username("me").build()?)
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "remove-external-account", "me"],
                )
                .await?;

                assert_eq!(output, "External account 'me' removed.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter.external_accounts().get("me").await?.is_none(),
                    "external account 'me' should not exist anymore"
                );

                Ok(())
            },
        )
        .await;
    }
}
