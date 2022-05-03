use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// show pull request info.
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub(crate) struct PullRequestShowCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,

    /// pull request number.
    #[argh(positional)]
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestShowCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let pr = CliDbExt::get_existing_pull_request(
            &mut *ctx.db_adapter.pull_requests(),
            owner,
            name,
            self.number,
        )
        .await?;

        println!(
            "Accessing pull request #{} on repository {}",
            self.number, self.repository_path
        );
        println!("{:#?}", pr);

        Ok(())
    }
}
