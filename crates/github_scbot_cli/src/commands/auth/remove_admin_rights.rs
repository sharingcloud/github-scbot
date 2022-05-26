use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_database2::Account;

use crate::commands::{Command, CommandContext};
use crate::errors::{DatabaseSnafu, IoSnafu};
use snafu::ResultExt;

/// remove admin rights from account.
#[derive(Parser)]
pub(crate) struct AuthRemoveAdminRightsCommand {
    /// account username.
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthRemoveAdminRightsCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut acc_db = ctx.db_adapter.accounts();
        match acc_db.get(&self.username).await.context(DatabaseSnafu)? {
            Some(_) => acc_db
                .set_is_admin(&self.username, false)
                .await
                .context(DatabaseSnafu)?,
            None => acc_db
                .create(
                    Account::builder()
                        .username(self.username.clone())
                        .is_admin(false)
                        .build()
                        .unwrap(),
                )
                .await
                .context(DatabaseSnafu)?,
        };

        writeln!(
            ctx.writer,
            "Account '{}' added/edited without admin rights.",
            self.username
        )
        .context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, Account, DbService, DbServiceImplPool};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_remove_admin_rights",
            |config, pool| async move {
                let api_adapter = MockApiService::new();
                let redis_adapter = MockRedisService::new();
                let db_adapter = DbServiceImplPool::new(pool.clone());

                db_adapter
                    .accounts()
                    .create(Account::builder().username("me").is_admin(true).build()?)
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["auth", "remove-admin-rights", "me"],
                )
                .await?;

                assert_eq!(output, "Account 'me' added/edited without admin rights.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    !db_adapter.accounts().get("me").await?.unwrap().is_admin(),
                    "account 'me' should not have admin rights"
                );

                Ok(())
            },
        )
        .await;
    }
}
