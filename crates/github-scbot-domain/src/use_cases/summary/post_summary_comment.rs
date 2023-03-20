use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::utils::sender::SummaryCommentSender;
use crate::{use_cases::status::PullRequestStatus, Result};

pub struct PostSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> PostSummaryCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
    ) -> Result<()> {
        SummaryCommentSender::create_or_update(
            self.api_service,
            self.db_service,
            self.lock_service,
            pr_handle,
            pr_status,
        )
        .await
        .map(|_| ())
    }
}
