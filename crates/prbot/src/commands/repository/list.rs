use async_trait::async_trait;
use clap::Parser;

use crate::{
    commands::{Command, CommandContext},
    Result,
};

/// List known repositories
#[derive(Parser)]
pub(crate) struct RepositoryListCommand;

#[async_trait]
impl Command for RepositoryListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let repos = ctx.db_service.repositories_all().await?;
        if repos.is_empty() {
            writeln!(ctx.writer.write().await, "No repository known.")?;
        } else {
            for repo in repos {
                writeln!(ctx.writer.write().await, "- {}/{}", repo.owner, repo.name)?;
            }
        }

        Ok(())
    }
}
