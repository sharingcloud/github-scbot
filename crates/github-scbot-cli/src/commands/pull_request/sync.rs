use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain::use_cases::pulls::SynchronizePullRequestAndUpdateStatusUseCase;
use github_scbot_domain_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    Result,
};

/// Synchronize pull request from upstream
#[derive(Debug, Parser)]
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

        SynchronizePullRequestAndUpdateStatusUseCase {
            api_service: ctx.api_service.as_ref(),
            config: &ctx.config,
            db_service: ctx.db_service.as_mut(),
            lock_service: ctx.lock_service.as_ref(),
            pr_number: self.number,
            repo_name,
            repo_owner,
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
