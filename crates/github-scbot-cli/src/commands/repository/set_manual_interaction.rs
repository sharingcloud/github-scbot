use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set manual interaction mode for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetManualInteractionCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Manual interaction mode
    #[clap(parse(try_from_str))]
    manual_interaction: bool,
}

#[async_trait(?Send)]
impl Command for RepositorySetManualInteractionCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_adapter.as_mut(), owner, name).await?;

        ctx.db_adapter
            .repositories_set_manual_interaction(owner, name, self.manual_interaction)
            .await?;

        writeln!(
            ctx.writer,
            "Manual interaction mode set to '{}' for repository {}.",
            self.manual_interaction, self.repository_path
        )?;

        Ok(())
    }
}
