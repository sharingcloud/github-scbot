use async_trait::async_trait;
use clap::Parser;
use prbot_models::{MergeStrategy, RepositoryPath};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Set merge strategy for a pull request
#[derive(Parser)]
pub(crate) struct PullRequestSetMergeStrategyCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,

    /// Merge strategy
    strategy: Option<MergeStrategy>,
}

#[async_trait]
impl Command for PullRequestSetMergeStrategyCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let _pr =
            CliDbExt::get_existing_pull_request(ctx.db_service.as_ref(), owner, name, self.number)
                .await?;
        ctx.db_service
            .pull_requests_set_strategy_override(owner, name, self.number, self.strategy)
            .await?;

        if let Some(s) = self.strategy {
            writeln!(
                ctx.writer.write().await,
                "Setting '{}' as a merge strategy override for pull request #{} on repository '{}'.",
                s, self.number, self.repository_path
            )?;
        } else {
            writeln!(
                ctx.writer.write().await,
                "Removing merge strategy override for pull request #{} on repository '{}'.",
                self.number,
                self.repository_path
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{PullRequest, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run_set() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["pull-requests", "set-merge-strategy", "owner/name", "1", "squash"]).await,
            "Setting 'squash' as a merge strategy override for pull request #1 on repository 'owner/name'.\n"
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_unset() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        assert_eq!(
            test_command(
                ctx,
                &["pull-requests", "set-merge-strategy", "owner/name", "1"]
            )
            .await,
            "Removing merge strategy override for pull request #1 on repository 'owner/name'.\n"
        );

        Ok(())
    }
}
