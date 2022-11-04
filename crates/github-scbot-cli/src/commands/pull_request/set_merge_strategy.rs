use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::{pulls::GhMergeStrategy, repository::RepositoryPath};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// List known pull request for a repository
#[derive(Parser)]
pub(crate) struct PullRequestSetMergeStrategyCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,

    /// Merge strategy
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
            .await?;

        if let Some(s) = self.strategy {
            writeln!(
                ctx.writer,
                "Setting '{}' as a merge strategy override for pull request #{} on repository '{}'.",
                s, self.number, self.repository_path
            )?;
        } else {
            writeln!(
                ctx.writer,
                "Removing merge strategy override for pull request #{} on repository '{}'.",
                self.number, self.repository_path
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::types::pulls::GhMergeStrategy;
    use github_scbot_database::{MockPullRequestDB, PullRequest};
    use mockall::predicate;

    use crate::testutils::{test_command, CommandContextTest};
    use crate::Result;

    #[actix_rt::test]
    async fn test_set() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                )
                .returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

            mock.expect_set_strategy_override()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(Some(GhMergeStrategy::Squash)),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });

            Box::new(mock)
        });

        let output = test_command(
            ctx,
            &[
                "pull-requests",
                "set-merge-strategy",
                "owner/name",
                "1",
                "squash",
            ],
        )
        .await?;
        assert_eq!(output, "Setting 'squash' as a merge strategy override for pull request #1 on repository 'owner/name'.\n");

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unset() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                )
                .returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

            mock.expect_set_strategy_override()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(None),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });

            Box::new(mock)
        });

        let output = test_command(
            ctx,
            &["pull-requests", "set-merge-strategy", "owner/name", "1"],
        )
        .await?;
        assert_eq!(
            output,
            "Removing merge strategy override for pull request #1 on repository 'owner/name'.\n"
        );

        Ok(())
    }
}
