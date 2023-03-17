use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain_models::Repository;

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

        writeln!(ctx.writer, "Repository {} created.", self.repository_path)?;
        Ok(())
    }
}
