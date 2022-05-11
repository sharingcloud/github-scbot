use std::io::Write;

use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::repository::RepositoryPath;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::ResultExt;

/// set manual interaction mode for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-manual-interaction")]
pub(crate) struct RepositorySetManualInteractionCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// mode.
    #[argh(positional)]
    manual_interaction: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetManualInteractionCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_manual_interaction(owner, name, self.manual_interaction)
            .await
            .context(DatabaseSnafu)?;

        writeln!(
            ctx.writer,
            "Manual interaction mode set to '{}' for repository {}.",
            self.manual_interaction, self.repository_path
        )
        .context(IoSnafu)?;

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
            "test_command_repository_set_manual_interaction",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(
                        Repository::builder()
                            .owner("owner")
                            .name("name")
                            .manual_interaction(false)
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &[
                        "repositories",
                        "set-manual-interaction",
                        "owner/name",
                        "true",
                    ],
                )
                .await?;

                assert_eq!(
                    output,
                    "Manual interaction mode set to 'true' for repository owner/name.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter
                        .repositories()
                        .get("owner", "name")
                        .await?
                        .unwrap()
                        .manual_interaction(),
                    "repository owner/name should have manual interaction set"
                );

                Ok(())
            },
        )
        .await;
    }
}
