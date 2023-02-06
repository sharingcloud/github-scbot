use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::{pulls::PullRequestLogic, status::StatusLogic};

use crate::commands::{Command, CommandContext};

/// Synchronize pull request from upstream
#[derive(Parser)]
pub(crate) struct PullRequestSyncCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestSyncCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (repo_owner, repo_name) = self.repository_path.components();
        let pr_number = self.number;

        PullRequestLogic::synchronize_pull_request(
            &ctx.config,
            ctx.db_adapter.as_mut(),
            repo_owner,
            repo_name,
            pr_number,
        )
        .await?;

        let upstream_pr = ctx
            .api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?;

        StatusLogic::update_pull_request_status(
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_mut(),
            ctx.redis_adapter.as_ref(),
            repo_owner,
            repo_name,
            pr_number,
            &upstream_pr,
        )
        .await?;

        writeln!(
            ctx.writer,
            "Pull request #{} from '{}' updated from GitHub.",
            self.number, self.repository_path
        )?;
        Ok(())
    }
}
