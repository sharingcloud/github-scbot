use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Set default automerge status for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultAutomergeCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Status
    #[clap(value_parser)]
    status: bool,
}

#[async_trait]
impl Command for RepositorySetDefaultAutomergeCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        ctx.db_service
            .repositories_set_default_automerge(owner, name, self.status)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Default automerge set to '{}' for repository {}.",
            self.status,
            self.repository_path
        )?;

        Ok(())
    }
}
