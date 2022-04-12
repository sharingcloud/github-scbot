use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// set PR title regex for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-title-regex")]
pub(crate) struct RepositorySetTitleRegexCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// regex value.
    #[argh(positional)]
    value: String,
}

#[async_trait(?Send)]
impl Command for RepositorySetTitleRegexCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let _repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        pr_repo
            .set_pr_title_validation_regex(owner, name, &self.value)
            .await?;

        println!(
            "PR title regular expression set to '{}' for repository '{}'.",
            self.value, self.repository_path
        );

        Ok(())
    }
}
