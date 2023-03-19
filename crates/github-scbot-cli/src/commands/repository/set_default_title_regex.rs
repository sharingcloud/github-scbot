use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Set PR title regex for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetDefaultTitleRegexCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Regex value
    value: String,
}

#[async_trait(?Send)]
impl Command for RepositorySetDefaultTitleRegexCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

        ctx.db_service
            .repositories_set_pr_title_validation_regex(owner, name, &self.value)
            .await?;

        writeln!(
            ctx.writer,
            "PR title regular expression set to '{}' for repository '{}'.",
            self.value, self.repository_path
        )?;

        Ok(())
    }
}
