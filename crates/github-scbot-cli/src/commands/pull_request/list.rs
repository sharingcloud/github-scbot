use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// List known pull request for a repository
#[derive(Parser)]
pub(crate) struct PullRequestListCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for PullRequestListCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let prs = ctx.db_adapter.pull_requests_list(owner, name).await?;

        if prs.is_empty() {
            writeln!(
                ctx.writer,
                "No pull request found for repository '{}'.",
                self.repository_path
            )?;
        } else {
            for pr in prs {
                writeln!(ctx.writer, "- #{}", pr.number)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{DbServiceAll, PullRequest, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run_no_prs() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["pull-requests", "list", "owner/name"]).await,
            "No pull request found for repository 'owner/name'.\n"
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
        let repo = ctx
            .db_adapter
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_adapter
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
