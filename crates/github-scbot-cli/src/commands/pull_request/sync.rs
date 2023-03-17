use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::use_cases::{
    pulls::SynchronizePullRequestUseCase, status::UpdatePullRequestStatusUseCase,
};

use crate::{
    commands::{Command, CommandContext},
    Result,
};

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

        SynchronizePullRequestUseCase {
            config: &ctx.config,
            db_service: ctx.db_service.as_mut(),
            repo_owner,
            repo_name,
            pr_number,
        }
        .run()
        .await?;

        let upstream_pr = ctx
            .api_service
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?;

        UpdatePullRequestStatusUseCase {
            api_service: ctx.api_service.as_ref(),
            db_service: ctx.db_service.as_mut(),
            lock_service: ctx.lock_service.as_ref(),
            repo_owner,
            repo_name,
            pr_number,
            upstream_pr: &upstream_pr,
        }
        .run()
        .await?;

        writeln!(
            ctx.writer,
            "Pull request #{} from '{}' updated from GitHub.",
            self.number, self.repository_path
        )?;
        Ok(())
    }
}
