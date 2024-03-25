use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::repositories::RenameRepositoryInterface;
use prbot_models::RepositoryPath;
use shaku::HasComponent;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Rename repository
#[derive(Parser)]
pub(crate) struct RepositoryRenameCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// New repository path
    new_repository_path: RepositoryPath,
}

#[async_trait]
impl Command for RepositoryRenameCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let rename_uc: &dyn RenameRepositoryInterface = ctx.core_module.resolve_ref();
        rename_uc
            .run(
                &ctx.as_core_context(),
                self.repository_path.clone(),
                self.new_repository_path.clone(),
            )
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Repository '{}' successfully renamed to '{}'.",
            self.repository_path,
            self.new_repository_path
        )?;

        Ok(())
    }
}
