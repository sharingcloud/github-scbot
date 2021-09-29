use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// set default checks status for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-checks-status")]
pub(crate) struct RepositorySetChecksStatusCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// status.
    #[argh(positional)]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetChecksStatusCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        repo.default_enable_checks = self.status;
        ctx.db_adapter.repository().save(&mut repo).await?;

        println!(
            "Default checks status set to '{}' for repository {}.",
            self.status, self.repository_path
        );

        Ok(())
    }
}