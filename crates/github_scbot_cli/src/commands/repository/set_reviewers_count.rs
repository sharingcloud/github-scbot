use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// set default reviewers count for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-reviewers-count")]
pub(crate) struct RepositorySetReviewersCountCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// regex value.
    #[argh(positional)]
    reviewers_count: u64,
}

#[async_trait(?Send)]
impl Command for RepositorySetReviewersCountCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_default_needed_reviewers_count(owner, name, self.reviewers_count)
            .await?;

        println!(
            "Default reviewers count updated to {} for repository {}.",
            self.reviewers_count, self.repository_path
        );

        Ok(())
    }
}
