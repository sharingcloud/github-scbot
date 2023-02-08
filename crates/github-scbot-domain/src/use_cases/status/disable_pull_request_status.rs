use github_scbot_core::types::status::StatusState;
use github_scbot_database::DbServiceAll;
use github_scbot_ghapi::adapter::ApiService;

use crate::{summary::SummaryCommentSender, Result};

use super::generate_status_message::VALIDATION_STATUS_MESSAGE;

pub struct DisablePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbServiceAll,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> DisablePullRequestStatusUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        let sha = self
            .api_service
            .pulls_get(self.repo_owner, self.repo_name, self.pr_number)
            .await?
            .head
            .sha;

        self.api_service
            .commit_statuses_update(
                self.repo_owner,
                self.repo_name,
                &sha,
                StatusState::Success,
                VALIDATION_STATUS_MESSAGE,
                "Bot disabled.",
            )
            .await?;

        SummaryCommentSender::delete(
            self.api_service,
            self.db_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
        )
        .await
    }
}
