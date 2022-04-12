use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// set manual interaction mode for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-manual-interaction")]
pub(crate) struct RepositorySetManualInteractionCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// mode.
    #[argh(positional)]
    manual_interaction: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetManualInteractionCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_manual_interaction(owner, name, self.manual_interaction)
            .await?;

        println!(
            "Manual interaction mode set to '{}' for repository {}.",
            self.manual_interaction, self.repository_path
        );

        Ok(())
    }
}
