use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{types::GhPullRequest, ApiService};

use super::utils::PullRequestStatus;
use crate::Result;

pub struct BuildPullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
}

impl<'a> BuildPullRequestStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle), ret)]
    pub async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<PullRequestStatus> {
        PullRequestStatus::from_database(self.api_service, self.db_service, pr_handle, upstream_pr)
            .await
    }
}
