use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::PullRequestStatus, Result};

use super::utils::sender::SummaryCommentSender;

pub struct PostSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub redis_service: &'a dyn LockService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub pr_status: &'a PullRequestStatus,
}

impl<'a> PostSummaryCommentUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        SummaryCommentSender::create_or_update(
            self.api_service,
            self.db_service,
            self.redis_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
            self.pr_status,
        )
        .await
        .map(|_| ())
    }
}
