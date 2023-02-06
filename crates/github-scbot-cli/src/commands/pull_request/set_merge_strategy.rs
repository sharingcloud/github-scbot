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

        let _pr =
            CliDbExt::get_existing_pull_request(ctx.db_adapter.as_mut(), owner, name, self.number)
                .await?;
        ctx.db_adapter
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
