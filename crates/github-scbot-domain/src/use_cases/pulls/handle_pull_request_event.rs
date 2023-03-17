use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{
    types::{GhPullRequestAction, GhPullRequestEvent},
    ApiService,
};
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandlePullRequestEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub event: GhPullRequestEvent,
}

impl<'a> HandlePullRequestEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?self.event.action,
            pr_number = self.event.number,
            repository_path = %self.event.repository.full_name,
            username = %self.event.pull_request.user.login
        )
    )]
    pub async fn run(&mut self) -> Result<()> {
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;

        let pr_model = match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, self.event.pull_request.number)
            .await?
        {
            Some(pr) => pr,
            None => return Ok(()),
        };

        let pr_number = pr_model.number;
        let mut status_changed = false;

        // Status update
        match self.event.action {
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

        if let GhPullRequestAction::Edited = self.event.action {
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
