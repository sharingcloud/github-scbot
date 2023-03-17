use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::{pulls::GhMergeStrategy, repository::RepositoryPath};

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
    strategy: Option<GhMergeStrategy>,
}

#[async_trait(?Send)]
impl Command for PullRequestSetMergeStrategyCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let _pr =
            CliDbExt::get_existing_pull_request(ctx.db_service.as_mut(), owner, name, self.number)
                .await?;
        ctx.db_service
            .pull_requests_set_strategy_override(owner, name, self.number, self.strategy)
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
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::{PullRequest, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run_set() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
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

    #[actix_rt::test]
    async fn run_unset() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
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
