use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::Repository;

use crate::commands::{Command, CommandContext};
use crate::errors::{DatabaseSnafu, IoSnafu};
use snafu::ResultExt;

/// Add repository
#[derive(Parser)]
pub(crate) struct RepositoryAddCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
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
            .build()
            .unwrap();

        ctx.db_adapter
            .repositories()
            .create(repo)
            .await
            .context(DatabaseSnafu)?;

        writeln!(ctx.writer, "Repository {} created.", self.repository_path).context(IoSnafu)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database::{use_temporary_db, DbService, DbServiceImplPool};
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
