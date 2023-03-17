use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Set default checks status for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultChecksStatusCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Status
    #[clap(parse(try_from_str))]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetDefaultChecksStatusCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

        ctx.db_service
            .repositories_set_default_enable_checks(owner, name, self.status)
            .await?;

        writeln!(
            ctx.writer,
            "Default checks status set to '{}' for repository {}.",
            self.status, self.repository_path
        )?;

        Ok(())
    }
}
