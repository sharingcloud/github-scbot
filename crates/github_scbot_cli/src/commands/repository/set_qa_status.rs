use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// set default QA status for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-qa-status")]
pub(crate) struct RepositorySetQAStatusCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// status.
    #[argh(positional)]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetQAStatusCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        repo.default_enable_qa = self.status;
        ctx.db_adapter.repository().save(&mut repo).await?;

        println!(
            "Default QA status set to '{}' for repository {}.",
            self.status, self.repository_path
        );

        Ok(())
    }
}
