use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// set default reviewers count for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-reviewers-count")]
pub(crate) struct RepositorySetReviewersCountCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// regex value.
    #[argh(positional)]
    reviewers_count: u32,
}

#[async_trait(?Send)]
impl Command for RepositorySetReviewersCountCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;

        repo.default_needed_reviewers_count = self.reviewers_count as i32;
        println!(
            "Default reviewers count updated to {} for repository {}.",
            self.reviewers_count, self.repository_path
        );
        ctx.db_adapter.repository().save(&mut repo).await?;

        Ok(())
    }
}
