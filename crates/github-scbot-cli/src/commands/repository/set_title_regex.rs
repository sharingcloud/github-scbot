use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::ResultExt;

/// Set PR title regex for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetTitleRegexCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Regex value
    value: String,
}

#[async_trait(?Send)]
impl Command for RepositorySetTitleRegexCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_pr_title_validation_regex(owner, name, &self.value)
            .await
            .context(DatabaseSnafu)?;

        writeln!(
            ctx.writer,
            "PR title regular expression set to '{}' for repository '{}'.",
            self.value, self.repository_path
        )
        .context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, Repository};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_set_title_regex",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(
                        Repository::builder()
                            .owner("owner")
                            .name("name")
                            .pr_title_validation_regex("")
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "set-title-regex", "owner/name", "[A-Z]+"],
                )
                .await?;

                assert_eq!(
                    output,
                    "PR title regular expression set to '[A-Z]+' for repository 'owner/name'.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert_eq!(
                    db_adapter
                        .repositories()
                        .get("owner", "name")
                        .await?
                        .unwrap()
                        .pr_title_validation_regex(),
                    "[A-Z]+",
                    "repository owner/name should have default needed reviewers to 10"
                );

                Ok(())
            },
        )
        .await;
    }
}
