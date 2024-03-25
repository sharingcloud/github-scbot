use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Show pull request info
#[derive(Parser)]
pub(crate) struct PullRequestShowCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,
}

#[async_trait]
impl Command for PullRequestShowCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let pr =
            CliDbExt::get_existing_pull_request(ctx.db_service.as_ref(), owner, name, self.number)
                .await?;

        writeln!(
            ctx.writer.write().await,
            "Accessing pull request #{} on repository '{}':",
            self.number,
            self.repository_path
        )?;
        writeln!(ctx.writer.write().await, "{:#?}", pr)?;

        Ok(())
    }
}
