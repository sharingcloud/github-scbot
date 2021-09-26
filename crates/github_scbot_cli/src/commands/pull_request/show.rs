use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// show pull request info.
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub(crate) struct PullRequestShowCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: String,

    /// pull request number.
    #[argh(positional)]
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestShowCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (pr, _repo) = ctx
            .db_adapter
            .pull_request()
            .get_from_repository_path_and_number(&self.repository_path, self.number)
            .await?;
        println!(
            "Accessing pull request #{} on repository {}",
            self.number, self.repository_path
        );
        println!("{:#?}", pr);

        Ok(())
    }
}
