use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain::use_cases::{
    checks::DetermineChecksStatusUseCase,
    pulls::{
        AutomergePullRequestUseCase, MergePullRequestUseCase, SetStepLabelUseCase,
        SynchronizePullRequestAndUpdateStatusUseCase, SynchronizePullRequestUseCase,
    },
    status::{BuildPullRequestStatusUseCase, UpdatePullRequestStatusUseCase},
    summary::UpdatePullRequestSummaryUseCase,
};
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
            synchronize_pull_request: &SynchronizePullRequestUseCase {
                config: &ctx.config,
                db_service: ctx.db_service.as_ref(),
            },
            update_pull_request_status: &UpdatePullRequestStatusUseCase {
                api_service: ctx.api_service.as_ref(),
                db_service: ctx.db_service.as_ref(),
                lock_service: ctx.lock_service.as_ref(),
                set_step_label: &SetStepLabelUseCase {
                    api_service: ctx.api_service.as_ref(),
                },
                automerge_pull_request: &AutomergePullRequestUseCase {
                    api_service: ctx.api_service.as_ref(),
                    db_service: ctx.db_service.as_ref(),
                    merge_pull_request: &MergePullRequestUseCase {
                        api_service: ctx.api_service.as_ref(),
                    },
                },
                update_pull_request_summary: &UpdatePullRequestSummaryUseCase {
                    api_service: ctx.api_service.as_ref(),
                },
                build_pull_request_status: &BuildPullRequestStatusUseCase {
                    api_service: ctx.api_service.as_ref(),
                    db_service: ctx.db_service.as_ref(),
                    determine_checks_status: &DetermineChecksStatusUseCase {
                        api_service: ctx.api_service.as_ref(),
                    },
                },
            },
        }
        .run(&(repo_owner, repo_name, self.number).into())
        .await?;

        writeln!(
            ctx.writer,
            "Pull request #{} from '{}' updated from GitHub.",
            self.number, self.repository_path
        )?;
        Ok(())
    }
}
