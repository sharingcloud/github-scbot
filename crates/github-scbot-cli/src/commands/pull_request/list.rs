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
                writeln!(ctx.writer, "- #{}", pr.number())?;
            }
        }

        Ok(())
    }
}
