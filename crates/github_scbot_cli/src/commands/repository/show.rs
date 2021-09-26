use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// show repository info.
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub(crate) struct RepositoryShowCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for RepositoryShowCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        println!("Accessing repository {}", self.repository_path);
        println!("{:#?}", repo);

        Ok(())
    }
}
