use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;

use super::utils::sender::SummaryCommentSender;
use crate::Result;

pub struct DeleteSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
}

impl<'a> DeleteSummaryCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        SummaryCommentSender::delete(self.api_service, self.db_service, pr_handle)
            .await
            .map(|_| ())
    }
}
