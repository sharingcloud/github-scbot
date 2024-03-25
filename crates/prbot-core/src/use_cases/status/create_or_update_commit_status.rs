use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequest;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use super::{utils::StatusMessageGenerator, PullRequestStatus};
use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait CreateOrUpdateCommitStatusInterface: Interface {
    async fn run<'b>(
        &self,
        ctx: &CoreContext<'b>,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
        upstream_pr: &GhPullRequest,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = CreateOrUpdateCommitStatusInterface)]
pub(crate) struct CreateOrUpdateCommitStatus;

#[async_trait]
impl CreateOrUpdateCommitStatusInterface for CreateOrUpdateCommitStatus {
    #[tracing::instrument(skip_all, fields(pr_handle,))]
    async fn run<'b>(
        &self,
        ctx: &CoreContext<'b>,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
        upstream_pr: &GhPullRequest,
    ) -> Result<()> {
        // Create or update status.
        let status_message = StatusMessageGenerator::default().generate(pr_status)?;
        ctx.api_service
            .commit_statuses_update(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                &upstream_pr.head.sha,
                status_message.state,
                status_message.title,
                &status_message.message,
            )
            .await?;

        Ok(())
    }
}
