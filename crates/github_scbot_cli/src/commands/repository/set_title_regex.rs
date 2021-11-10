use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use github_scbot_sentry::eyre::Result;

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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        println!("Accessing repository {}", self.repository_path);
        println!(
            "Setting value '{}' as PR title validation regex",
            self.value
        );

        let update = repo
            .create_update()
            .pr_title_validation_regex(&self.value)
            .build_update();
        ctx.db_adapter
            .repository()
            .update(&mut repo, update)
            .await?;

        Ok(())
    }
}
