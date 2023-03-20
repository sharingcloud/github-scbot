use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhCheckSuiteEvent, ApiService};
use github_scbot_lock_interface::LockService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandleCheckSuiteEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> HandleCheckSuiteEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            repository_path = %event.repository.full_name,
            head_branch = %event.check_suite.head_branch,
            head_sha = %event.check_suite.head_sha,
            app_slug = %event.check_suite.app.slug,
            status = ?event.check_suite.status,
            conclusion = ?event.check_suite.conclusion
        )
    )]
    pub async fn run(&self, event: GhCheckSuiteEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        // Only look for first PR
        if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
            let pr_number = gh_pr.number;

            if let Some(pr_model) = self
                .db_service
                .pull_requests_get(repo_owner, repo_name, pr_number)
                .await?
            {
                // Skip non Github Actions checks
                if event.check_suite.app.slug != "github-actions" {
                    return Ok(());
                }

                // Skip non up-to-date checks
                if event.check_suite.head_sha != gh_pr.head.sha {
                    return Ok(());
                }

                // Skip if checks are skipped
                if !pr_model.checks_enabled {
                    return Ok(());
                }

                let upstream_pr = self
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                // Update status
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
        }

        Ok(())
    }
}
