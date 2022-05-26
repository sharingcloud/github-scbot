use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_database2::ExternalAccount;

use crate::commands::{Command, CommandContext};
use crate::errors::{DatabaseSnafu, IoSnafu};
use snafu::ResultExt;

/// create external account.
#[derive(Parser)]
pub(crate) struct AuthCreateExternalAccountCommand {
    /// account username.
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthCreateExternalAccountCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();

        exa_db
            .create(
                ExternalAccount::builder()
                    .username(self.username.clone())
                    .generate_keys()
                    .build()
                    .unwrap(),
            )
            .await
            .context(DatabaseSnafu)?;

        writeln!(ctx.writer, "External account '{}' created.", self.username).context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_create_external_account",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "create-external-account", "me"],
                )
                .await?;

                assert_eq!(output, "External account 'me' created.\n");

                let db_adapter = DbServiceImplPool::new(pool);
                assert!(
                    db_adapter.external_accounts().get("me").await?.is_some(),
                    "external account 'me' should exist"
                );

                Ok(())
            },
        )
        .await;
    }
}
