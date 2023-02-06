use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::Repository;

use crate::commands::{Command, CommandContext};

/// Add repository
#[derive(Parser)]
pub(crate) struct RepositoryAddCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryAddCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let repo = Repository::builder()
            .owner(owner)
            .name(name)
            .with_config(&ctx.config)
            .build()
            .unwrap();

        ctx.db_adapter.repositories_create(repo).await?;

        writeln!(ctx.writer, "Repository {} created.", self.repository_path)?;
        Ok(())
    }
}
