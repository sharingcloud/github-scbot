use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// add repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
pub(crate) struct RepositoryAddCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for RepositoryAddCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) =
            RepositoryModel::extract_owner_and_name_from_path(&self.repository_path)?;
        RepositoryModel::builder(&ctx.config, owner, name)
            .create_or_update(ctx.db_adapter.repository())
            .await?;
        println!("Repository {} created.", self.repository_path);
        Ok(())
    }
}
