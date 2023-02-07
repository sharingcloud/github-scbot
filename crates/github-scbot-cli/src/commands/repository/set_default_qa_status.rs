use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set default QA status for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultQaStatusCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Status
    #[clap(parse(try_from_str))]
    status: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetDefaultQaStatusCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_adapter.as_mut(), owner, name).await?;

        ctx.db_adapter
            .repositories_set_default_enable_qa(owner, name, self.status)
            .await?;

        writeln!(
            ctx.writer,
            "Default QA status set to '{}' for repository {}.",
            self.status, self.repository_path
        )?;

        Ok(())
    }
}
