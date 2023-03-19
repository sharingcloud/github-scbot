use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhReviewEvent, ApiService};
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandleReviewEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub event: GhReviewEvent,
}

impl<'a> HandleReviewEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = self.event.repository.owner.login,
            repo_name = self.event.repository.name,
            pr_number = self.event.pull_request.number,
            reviewer = self.event.review.user.login,
            state = ?self.event.review.state
        )
    )]
    pub async fn run(&mut self) -> Result<()> {
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;
        let pr_number = self.event.pull_request.number;

        // Detect required reviews
        if self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
            .is_some()
        {
            let upstream_pr = self
                .api_service
                .pulls_get(repo_owner, repo_name, pr_number)
                .await?;

            UpdatePullRequestStatusUseCase {
                api_service: self.api_service,
                db_service: self.db_service,
                lock_service: self.lock_service,
                repo_name,
                repo_owner,
                pr_number,
                upstream_pr: &upstream_pr,
            }
            .run()
            .await?;
        }

        Ok(())
    }
}
