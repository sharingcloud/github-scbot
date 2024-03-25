use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::pulls::SynchronizePullRequestAndUpdateStatusInterface;
use prbot_models::RepositoryPath;
use shaku::HasComponent;

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

#[async_trait]
impl Command for PullRequestSyncCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (repo_owner, repo_name) = self.repository_path.components();

        let synchronize_and_update: &dyn SynchronizePullRequestAndUpdateStatusInterface =
            ctx.core_module.resolve_ref();
        synchronize_and_update
            .run(
                &ctx.as_core_context(),
                &(repo_owner, repo_name, self.number).into(),
            )
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Pull request #{} from '{}' updated from GitHub.",
            self.number,
            self.repository_path
        )?;
        Ok(())
    }
}
