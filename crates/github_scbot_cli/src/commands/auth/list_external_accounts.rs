use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;

use crate::commands::{Command, CommandContext};
use crate::errors::{DatabaseSnafu, IoSnafu};
use snafu::ResultExt;

/// List external accounts
#[derive(Parser)]
pub(crate) struct AuthListExternalAccountsCommand;

#[async_trait(?Send)]
impl Command for AuthListExternalAccountsCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let accounts = ctx
            .db_adapter
            .external_accounts()
            .all()
            .await
            .context(DatabaseSnafu)?;
        if accounts.is_empty() {
            writeln!(ctx.writer, "No external account found.").context(IoSnafu)?;
        } else {
            writeln!(ctx.writer, "External accounts:").context(IoSnafu)?;
            for account in accounts {
                writeln!(ctx.writer, "- {}", account.username()).context(IoSnafu)?;
            }
        }

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
            "test_command_list_external_accounts",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["auth", "list-external-accounts"],
                )
                .await?;

                assert_eq!(output, "No external account found.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .external_accounts()
                    .create(ExternalAccount::builder().username("me").build()?)
                    .await?;
                db_adapter
                    .external_accounts()
                    .create(ExternalAccount::builder().username("him").build()?)
                    .await?;
                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["auth", "list-external-accounts"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        External accounts:
                        - me
                        - him
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
