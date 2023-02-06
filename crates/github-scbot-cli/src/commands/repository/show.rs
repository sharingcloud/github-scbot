use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Show repository info
#[derive(Parser)]
pub(crate) struct RepositoryShowCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryShowCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_adapter.as_mut(), owner, name).await?;

        writeln!(ctx.writer, "Accessing repository {}", self.repository_path)?;
        writeln!(ctx.writer, "{:#?}", repo)?;

        Ok(())
    }
}
