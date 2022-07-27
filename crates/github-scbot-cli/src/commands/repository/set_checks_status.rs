use std::io::Write;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set default checks status for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetChecksStatusCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Status
    #[clap(parse(try_from_str))]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetChecksStatusCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_default_enable_checks(owner, name, self.status)
            .await
            .context(DatabaseSnafu)?;

        writeln!(
            ctx.writer,
            "Default checks status set to '{}' for repository {}.",
            self.status, self.repository_path
        )
        .context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database::{use_temporary_db, DbService, DbServiceImplPool, Repository};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_set_checks_status",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(
                        Repository::builder()
                            .owner("owner")
                            .name("name")
                            .default_enable_checks(true)
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "set-checks-status", "owner/name", "false"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Default checks status set to 'false' for repository owner/name.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    !db_adapter
                        .repositories()
                        .get("owner", "name")
                        .await?
                        .unwrap()
                        .default_enable_checks(),
                    "repository owner/name should have checks status to false"
                );

                Ok(())
            },
        )
        .await;
    }
}
