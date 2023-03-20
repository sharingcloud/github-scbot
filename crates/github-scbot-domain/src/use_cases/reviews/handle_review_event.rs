use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhReviewEvent, ApiService};
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandleReviewEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> HandleReviewEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = event.repository.owner.login,
            repo_name = event.repository.name,
            pr_number = event.pull_request.number,
            reviewer = event.review.user.login,
            state = ?event.review.state
        )
    )]
    pub async fn run(&self, event: GhReviewEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.pull_request.number;

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
            }
            .run(
                &(repo_owner.as_str(), repo_name.as_str(), pr_number).into(),
                &upstream_pr,
            )
            .await?;
        }

        Ok(())
    }
}
