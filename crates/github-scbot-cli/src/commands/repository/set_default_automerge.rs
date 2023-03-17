use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set default automerge status for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultAutomergeCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Status
    #[clap(parse(try_from_str))]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetDefaultAutomergeCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

        ctx.db_service
            .repositories_set_default_automerge(owner, name, self.status)
            .await?;

        writeln!(
            ctx.writer,
            "Default automerge set to '{}' for repository {}.",
            self.status, self.repository_path
        )?;

        Ok(())
    }
}
