use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain_models::RepositoryPath;

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

#[async_trait(?Send)]
impl Command for PullRequestShowCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let pr =
            CliDbExt::get_existing_pull_request(ctx.db_service.as_mut(), owner, name, self.number)
                .await?;

        writeln!(
            ctx.writer,
            "Accessing pull request #{} on repository '{}':",
            self.number, self.repository_path
        )?;
        writeln!(ctx.writer, "{:#?}", pr)?;

        Ok(())
    }
}
