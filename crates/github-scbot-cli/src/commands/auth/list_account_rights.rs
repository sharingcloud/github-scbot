use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// List rights for account
#[derive(Parser)]
pub(crate) struct AuthListAccountRightsCommand {
    /// Account username
    username: String,
}

#[async_trait(?Send)]
impl Command for AuthListAccountRightsCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let mut repo_db = ctx.db_adapter.repositories();
        let mut exa_db = ctx.db_adapter.external_accounts();
        let mut exr_db = ctx.db_adapter.external_account_rights();

        let _exa = CliDbExt::get_existing_external_account(&mut *exa_db, &self.username).await?;
        let rights = exr_db.list(&self.username).await?;

        if rights.is_empty() {
            writeln!(
                ctx.writer,
                "No right found from account '{}'.",
                self.username
            )?;
        } else {
            writeln!(ctx.writer, "Rights from account '{}':", self.username)?;
            for right in rights {
                let repo = repo_db.get_from_id(right.repository_id()).await?.unwrap();
                writeln!(ctx.writer, "- {}/{}", repo.owner(), repo.name())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database::{
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
            "test_command_list_account_rights",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let repo1 = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;
                let repo2 = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name2").build()?)
                    .await?;
                db_adapter
                    .external_accounts()
                    .create(ExternalAccount::builder().username("me").build()?)
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["auth", "list-account-rights", "me"],
                )
                .await?;

                assert_eq!(output, "No right found from account 'me'.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .external_account_rights()
                    .create(
                        ExternalAccountRight::builder()
                            .with_repository(&repo2)
                            .username("me")
                            .build()?,
                    )
                    .await?;
                db_adapter
                    .external_account_rights()
                    .create(
                        ExternalAccountRight::builder()
                            .with_repository(&repo1)
                            .username("me")
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["auth", "list-account-rights", "me"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        Rights from account 'me':
                        - owner/name
                        - owner/name2
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
