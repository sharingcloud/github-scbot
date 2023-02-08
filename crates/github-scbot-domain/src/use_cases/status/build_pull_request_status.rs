use github_scbot_core::types::pulls::GhPullRequest;
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;

use super::utils::PullRequestStatus;
use crate::Result;

pub struct BuildPullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub upstream_pr: &'a GhPullRequest,
}

impl<'a> BuildPullRequestStatusUseCase<'a> {
    pub async fn run(&mut self) -> Result<PullRequestStatus> {
        PullRequestStatus::from_database(
            self.api_service,
            self.db_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
            self.upstream_pr,
        )
        .await
    }
}
