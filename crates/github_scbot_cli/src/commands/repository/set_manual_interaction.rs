use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// set manual interaction mode for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-manual-interaction")]
pub(crate) struct RepositorySetManualInteractionCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// mode.
    #[argh(positional)]
    manual_interaction: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetManualInteractionCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        repo.manual_interaction = self.manual_interaction;
        ctx.db_adapter.repository().save(&mut repo).await?;

        println!(
            "Manual interaction mode set to '{}' for repository {}.",
            self.manual_interaction, self.repository_path
        );

        Ok(())
    }
}