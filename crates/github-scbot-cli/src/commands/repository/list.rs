use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;

use crate::commands::{Command, CommandContext};

/// List known repositories
#[derive(Parser)]
pub(crate) struct RepositoryListCommand;

#[async_trait(?Send)]
impl Command for RepositoryListCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let repos = ctx.db_adapter.repositories_all().await?;
        if repos.is_empty() {
            writeln!(ctx.writer, "No repository known.")?;
        } else {
            for repo in repos {
                writeln!(ctx.writer, "- {}/{}", repo.owner(), repo.name())?;
            }
        }

        Ok(())
    }
}
