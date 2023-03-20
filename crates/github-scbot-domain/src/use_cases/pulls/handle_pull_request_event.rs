use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{
    types::{GhPullRequestAction, GhPullRequestEvent},
    ApiService,
};
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandlePullRequestEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> HandlePullRequestEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            pr_number = event.number,
            repository_path = %event.repository.full_name,
            username = %event.pull_request.user.login
        )
    )]
    pub async fn run(&self, event: GhPullRequestEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        let pr_model = match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, event.pull_request.number)
            .await?
        {
            Some(pr) => pr,
            None => return Ok(()),
        };

        let pr_number = pr_model.number;
        let mut status_changed = false;

        // Status update
        match event.action {
            GhPullRequestAction::Synchronize => {
                // Force status to waiting
                status_changed = true;
            }
            GhPullRequestAction::Reopened
            | GhPullRequestAction::ReadyForReview
            | GhPullRequestAction::ConvertedToDraft
            | GhPullRequestAction::Closed => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequested => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequestRemoved => {
                status_changed = true;
            }
            _ => (),
        }

        if let GhPullRequestAction::Edited = event.action {
            // Update PR title
            status_changed = true;
        }

        if status_changed {
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
