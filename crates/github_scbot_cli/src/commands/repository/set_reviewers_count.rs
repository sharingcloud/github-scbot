use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;

        let update = repo
            .create_update()
            .default_needed_reviewers_count(self.reviewers_count as u64)
            .build_update();
        ctx.db_adapter
            .repository()
            .update(&mut repo, update)
            .await?;

        println!(
            "Default reviewers count updated to {} for repository {}.",
            self.reviewers_count, self.repository_path
        );

        Ok(())
    }
}
