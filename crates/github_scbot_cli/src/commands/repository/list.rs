use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list known repositories.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub(crate) struct RepositoryListCommand {}

#[async_trait(?Send)]
impl Command for RepositoryListCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let repos = ctx.db_adapter.repositories().all().await?;
        if repos.is_empty() {
            writeln!(ctx.writer, "No repository known.")?;
        } else {
            for repo in repos {
                writeln!(ctx.writer, "- {}/{}", repo.owner(), repo.name())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, Repository};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_list",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "list"],
                )
                .await?;

                assert_eq!(output, "No repository known.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;
                db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name2").build()?)
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "list"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
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
