use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// create external token.
#[derive(FromArgs)]
#[argh(subcommand, name = "create-external-token")]
pub(crate) struct AuthCreateExternalTokenCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthCreateExternalTokenCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut exa_db = ctx.db_adapter.external_accounts();
        let exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        writeln!(ctx.writer, "{}", exa.generate_access_token()?)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, ExternalAccount};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_create_external_token",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                db_adapter
                    .external_accounts()
                    .create(
                        ExternalAccount::builder()
                            .username("me")
                            .generate_keys()
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "create-external-token", "me"],
                )
                .await?;

                assert!(output.starts_with("ey"), "JWTs should begin with 'ey'");

                Ok(())
            },
        )
        .await;
    }
}
