use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Repository;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// add repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
pub(crate) struct RepositoryAddCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryAddCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let repo = Repository::builder()
            .owner(owner.into())
            .name(name.into())
            .build()?;

        ctx.db_adapter.repositories().create(repo).await?;

        println!("Repository {} created.", self.repository_path);
        Ok(())
    }
}
