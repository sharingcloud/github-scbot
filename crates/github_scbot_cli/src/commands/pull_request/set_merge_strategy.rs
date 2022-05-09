use std::io::Write;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::{pulls::GhMergeStrategy, repository::RepositoryPath};
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-merge-strategy")]
pub(crate) struct PullRequestSetMergeStrategyCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,

    /// pull request number.
    #[argh(positional)]
    number: u64,

    /// merge strategy.
    #[argh(positional)]
    strategy: Option<GhMergeStrategy>,
}

#[async_trait(?Send)]
impl Command for PullRequestSetMergeStrategyCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut pr_db = ctx.db_adapter.pull_requests();
        let _pr =
            CliDbExt::get_existing_pull_request(&mut *pr_db, owner, name, self.number).await?;
        pr_db
            .set_strategy_override(owner, name, self.number, self.strategy)
            .await
            .context(DatabaseSnafu)?;

        if let Some(s) = self.strategy {
            writeln!(
                ctx.writer,
                "Setting {:?} as a merge strategy override for pull request #{} on repository {}",
                s, self.number, self.repository_path
            )
            .context(IoSnafu)?;
        } else {
            writeln!(
                ctx.writer,
                "Removing merge strategy override for pull request #{} on repository {}",
                self.number, self.repository_path
            )
            .context(IoSnafu)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{
        use_temporary_db, DbService, DbServiceImplPool, PullRequest, Repository,
    };
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;
    use github_scbot_types::pulls::GhMergeStrategy;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_pull_request_set_merge_strategy",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                let repo = db_adapter
                    .repositories()
                    .create(
                        Repository::builder()
                        .owner("owner")
                        .name("name")
                        .default_strategy(GhMergeStrategy::Merge)
                        .build()?
                    )
                    .await?;

                let pr = db_adapter
                    .pull_requests()
                    .create(PullRequest::builder().with_repository(&repo).number(1u64).build()?)
                    .await?;
                assert!(pr.strategy_override().is_none(), "no strategy override should be set");

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["pull-requests", "set-merge-strategy", "owner/name", "1", "squash"],
                )
                .await?;

                let db_adapter = DbServiceImplPool::new(pool.clone());
                let pr = db_adapter.pull_requests().get("owner", "name", 1).await?.unwrap();
                assert_eq!(*pr.strategy_override(), Some(GhMergeStrategy::Squash));
                assert_eq!(output, "Setting Squash as a merge strategy override for pull request #1 on repository owner/name\n");

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["pull-requests", "set-merge-strategy", "owner/name", "1"],
                )
                .await?;

                let db_adapter = DbServiceImplPool::new(pool.clone());
                let pr = db_adapter.pull_requests().get("owner", "name", 1).await?.unwrap();
                assert_eq!(*pr.strategy_override(), None);
                assert_eq!(output, "Removing merge strategy override for pull request #1 on repository owner/name\n");

                Ok(())
            },
        )
        .await;
    }
}
