use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    Result,
};

/// List known pull request for a repository
#[derive(Parser)]
pub(crate) struct PullRequestListCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,
}

#[async_trait]
impl Command for PullRequestListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let prs = ctx.db_service.pull_requests_list(owner, name).await?;

        if prs.is_empty() {
            writeln!(
                ctx.writer.write().await,
                "No pull request found for repository '{}'.",
                self.repository_path
            )?;
        } else {
            for pr in prs {
                writeln!(ctx.writer.write().await, "- #{}", pr.number)?;
            }
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
    async fn run_no_prs() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["pull-requests", "list", "owner/name"]).await,
            "No pull request found for repository 'owner/name'.\n"
        );

        Ok(())
    }

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
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
            test_command(ctx, &["pull-requests", "list", "owner/name"]).await,
            "- #1\n"
        );

        Ok(())
    }
}
