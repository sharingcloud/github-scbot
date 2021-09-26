use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use github_scbot_logic::{pulls::synchronize_pull_request, status::update_pull_request_status};
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// synchronize pull request from upstream.
#[derive(FromArgs)]
#[argh(subcommand, name = "sync")]
pub(crate) struct PullRequestSyncCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: String,

    /// pull request number.
    #[argh(positional)]
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestSyncCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) =
            RepositoryModel::extract_owner_and_name_from_path(&self.repository_path)?;
        let (mut pr, sha) = synchronize_pull_request(
            &ctx.config,
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            owner,
            name,
            self.number,
        )
        .await?;
        let repo = ctx
            .db_adapter
            .repository()
            .get_from_owner_and_name(owner, name)
            .await?;
        update_pull_request_status(
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            ctx.redis_adapter.as_ref(),
            &repo,
            &mut pr,
            &sha,
        )
        .await?;

        println!(
            "Pull request #{} from {} updated from GitHub.",
            self.number, self.repository_path
        );
        Ok(())
    }
}
