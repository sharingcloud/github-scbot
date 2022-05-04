use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Account;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// add admin rights to account.
#[derive(FromArgs)]
#[argh(subcommand, name = "add-admin-rights")]
pub(crate) struct AuthAddAdminRightsCommand {
    /// account username.
    #[argh(positional)]
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthAddAdminRightsCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut acc_db = ctx.db_adapter.accounts();
        match acc_db.get(&self.username).await? {
            Some(_) => acc_db.set_is_admin(&self.username, true).await?,
            None => {
                acc_db
                    .create(
                        Account::builder()
                            .username(self.username.clone())
                            .is_admin(true)
                            .build()?,
                    )
                    .await?
            }
        };

        writeln!(
            ctx.writer,
            "Account '{}' added/edited with admin rights.",
            self.username
        )?;

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
            "test_command_add_admin_rights",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "add-admin-rights", "me"],
                )
                .await?;

                assert_eq!(output, "Account 'me' added/edited with admin rights.\n");

                let db_adapter = DbServiceImplPool::new(pool);
                assert!(
                    db_adapter.accounts().get("me").await?.unwrap().is_admin(),
                    "account 'me' should exist and should be admin"
                );

                Ok(())
            },
        )
        .await;
    }
}
