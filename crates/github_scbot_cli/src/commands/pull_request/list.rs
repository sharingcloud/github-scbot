use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub(crate) struct PullRequestListCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for PullRequestListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let prs = ctx.db_adapter.pull_requests().list(owner, name).await?;

        if prs.is_empty() {
            println!("No PR found from repository '{}'.", self.repository_path);
        } else {
            for pr in prs {
                println!("- #{}", pr.number());
            }
        }

        Ok(())
    }
}
