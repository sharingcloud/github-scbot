use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Repository;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// add repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
pub(crate) struct RepositoryAddCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryAddCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let repo = Repository::builder()
            .owner(owner)
            .name(name)
            .with_config(&ctx.config)
            .build()?;

        ctx.db_adapter.repositories().create(repo).await?;

        writeln!(ctx.writer, "Repository {} created.", self.repository_path)?;
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
            "test_command_repository_add",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "add", "owner/name"],
                )
                .await?;

                assert_eq!(output, "Repository owner/name created.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter
                        .repositories()
                        .get("owner", "name")
                        .await?
                        .is_some(),
                    "repository owner/name should exist"
                );

                Ok(())
            },
        )
        .await;
    }
}
