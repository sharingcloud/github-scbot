use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set default reviewers count for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultReviewersCountCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Regex value
    reviewers_count: u64,
}

#[async_trait(?Send)]
impl Command for RepositorySetDefaultReviewersCountCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

        ctx.db_service
            .repositories_set_default_needed_reviewers_count(owner, name, self.reviewers_count)
            .await?;

        writeln!(
            ctx.writer,
            "Default reviewers count updated to {} for repository {}.",
            self.reviewers_count, self.repository_path
        )?;

        Ok(())
    }
}
