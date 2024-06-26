use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Set manual interaction mode for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetManualInteractionCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Manual interaction mode
    #[clap(value_parser)]
    manual_interaction: bool,
}

#[async_trait]
impl Command for RepositorySetManualInteractionCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        ctx.db_service
            .repositories_set_manual_interaction(owner, name, self.manual_interaction)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Manual interaction mode set to '{}' for repository {}.",
            self.manual_interaction,
            self.repository_path
        )?;

        Ok(())
    }
}
