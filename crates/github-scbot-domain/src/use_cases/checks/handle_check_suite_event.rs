use github_scbot_core::types::checks::GhCheckSuiteEvent;
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct HandleCheckSuiteEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub redis_service: &'a dyn RedisService,
    pub event: GhCheckSuiteEvent,
}

impl<'a> HandleCheckSuiteEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?self.event.action,
            repository_path = %self.event.repository.full_name,
            head_branch = %self.event.check_suite.head_branch,
            head_sha = %self.event.check_suite.head_sha,
            app_slug = %self.event.check_suite.app.slug,
            status = ?self.event.check_suite.status,
            conclusion = ?self.event.check_suite.conclusion
        )
    )]
    pub async fn run(&mut self) -> Result<()> {
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;

        // Only look for first PR
        if let Some(gh_pr) = self.event.check_suite.pull_requests.get(0) {
            let pr_number = gh_pr.number;

            if let Some(pr_model) = self
                .db_service
                .pull_requests_get(repo_owner, repo_name, pr_number)
                .await?
            {
                // Skip non Github Actions checks
                if self.event.check_suite.app.slug != "github-actions" {
                    return Ok(());
                }

                // Skip non up-to-date checks
                if self.event.check_suite.head_sha != gh_pr.head.sha {
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
                    redis_service: self.redis_service,
                    repo_name,
                    repo_owner,
                    pr_number,
                    upstream_pr: &upstream_pr,
                }
                .run()
                .await?;
            }
        }

        Ok(())
    }
}
