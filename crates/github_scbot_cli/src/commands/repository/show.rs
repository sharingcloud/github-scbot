use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// show repository info.
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub(crate) struct RepositoryShowCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryShowCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        println!("Accessing repository {}", self.repository_path);
        println!("{:#?}", repo);

        Ok(())
    }
}
