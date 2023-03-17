use github_scbot_core::types::status::StatusState;
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::ApiService;

use super::generate_status_message::VALIDATION_STATUS_MESSAGE;
use crate::{use_cases::summary::DeleteSummaryCommentUseCase, Result};

pub struct DisablePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
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

        DeleteSummaryCommentUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            pr_number: self.pr_number,
            repo_name: self.repo_name,
            repo_owner: self.repo_owner,
        }
        .run()
        .await
    }
}
