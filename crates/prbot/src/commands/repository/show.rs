use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Show repository info
#[derive(Parser)]
pub(crate) struct RepositoryShowCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait]
impl Command for RepositoryShowCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        writeln!(
            ctx.writer.write().await,
            "Accessing repository {}",
            self.repository_path
        )?;
        writeln!(ctx.writer.write().await, "{:#?}", repo)?;

        Ok(())
    }
}
