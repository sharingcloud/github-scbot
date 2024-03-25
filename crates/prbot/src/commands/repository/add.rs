use async_trait::async_trait;
use clap::Parser;
use prbot_models::{Repository, RepositoryPath};

use crate::{
    commands::{Command, CommandContext},
    Result,
};

/// Add repository
#[derive(Parser)]
pub(crate) struct RepositoryAddCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait]
impl Command for RepositoryAddCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        ctx.db_service
            .repositories_create(
                Repository {
                    owner: owner.to_owned(),
                    name: name.to_owned(),
                    ..Default::default()
                }
                .with_config(&ctx.config),
            )
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Repository {} created.",
            self.repository_path
        )?;
        Ok(())
    }
}
