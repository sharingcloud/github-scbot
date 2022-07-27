//! Checks logic.

use github_scbot_core::types::checks::GhCheckSuiteEvent;
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use crate::{status::StatusLogic, Result};

/// Handle GitHub check syite event.
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
pub async fn handle_check_suite_event(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhCheckSuiteEvent,
) -> Result<()> {
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;

    // Only look for first PR
    if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
        let pr_number = gh_pr.number;

        if let Some(pr_model) = db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
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
            if !pr_model.checks_enabled() {
                return Ok(());
            }

            let upstream_pr = api_adapter
                .pulls_get(repo_owner, repo_name, pr_number)
                .await?;

            // Update status
            StatusLogic::update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                repo_owner,
                repo_name,
                pr_number,
                &upstream_pr,
            )
            .await?;
        }
    }

    Ok(())
}