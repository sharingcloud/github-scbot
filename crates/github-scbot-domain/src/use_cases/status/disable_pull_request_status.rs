use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{types::GhCommitStatus, ApiService};

use super::utils::VALIDATION_STATUS_MESSAGE;
use crate::{use_cases::summary::DeleteSummaryCommentUseCase, Result};

pub struct DisablePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
}

impl<'a> DisablePullRequestStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        let sha = self
            .api_service
            .pulls_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .head
            .sha;

        self.api_service
            .commit_statuses_update(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                &sha,
                GhCommitStatus::Success,
                VALIDATION_STATUS_MESSAGE,
                "Bot disabled.",
            )
            .await?;

        DeleteSummaryCommentUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
        }
        .run(pr_handle)
        .await
    }
}
