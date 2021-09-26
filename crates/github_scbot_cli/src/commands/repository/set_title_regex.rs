use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// set PR title regex for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-title-regex")]
pub(crate) struct RepositorySetTitleRegexCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// regex value.
    #[argh(positional)]
    value: String,
}

#[async_trait(?Send)]
impl Command for RepositorySetTitleRegexCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        println!("Accessing repository {}", self.repository_path);
        println!(
            "Setting value '{}' as PR title validation regex",
            self.value
        );
        repo.pr_title_validation_regex = self.value.to_owned();
        ctx.db_adapter.repository().save(&mut repo).await?;

        Ok(())
    }
}
