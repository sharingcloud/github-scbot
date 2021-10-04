use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub(crate) struct PullRequestListCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for PullRequestListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let prs = ctx
            .db_adapter
            .pull_request()
            .list_from_repository_path(&self.repository_path)
            .await?;
        if prs.is_empty() {
            println!("No PR found from repository '{}'.", self.repository_path);
        } else {
            for pr in prs {
                println!("- #{}: {}", pr.number(), pr.name());
            }
        }

        Ok(())
    }
}
