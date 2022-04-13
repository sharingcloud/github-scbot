use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_logic::{pulls::PullRequestLogic, status::StatusLogic};
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// synchronize pull request from upstream.
#[derive(FromArgs)]
#[argh(subcommand, name = "sync")]
pub(crate) struct PullRequestSyncCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,

    /// pull request number.
    #[argh(positional)]
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestSyncCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let (pr, _sha) = PullRequestLogic::synchronize_pull_request(
            &ctx.config,
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            owner,
            name,
            self.number,
        )
        .await?;

        let upstream_pr = ctx.api_adapter.pulls_get(owner, name, self.number).await?;

        let repo = ctx
            .db_adapter
            .repositories()
            .get(owner, name)
            .await?
            .unwrap();

        StatusLogic::update_pull_request_status(
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            ctx.redis_adapter.as_ref(),
            &repo,
            &pr,
            &upstream_pr
        )
        .await?;

        println!(
            "Pull request #{} from {} updated from GitHub.",
            self.number, self.repository_path
        );
        Ok(())
    }
}
