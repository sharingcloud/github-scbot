//! Checks logic.

use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::checks::GhCheckSuiteEvent;

use crate::{status::StatusLogic, Result};

/// Handle GitHub check syite event.
pub async fn handle_check_suite_event(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhCheckSuiteEvent,
) -> Result<()> {
    let owner = &event.repository.owner.login;
    let name = &event.repository.name;

    // Only look for first PR
    if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
        if let Some(pr_model) = db_adapter
            .pull_requests()
            .get(owner, name, gh_pr.number)
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

            // Unwrap: Repo should exist.
            let repo_model = db_adapter.repositories().get(owner, name).await?.unwrap();
            let upstream_pr = api_adapter
                .pulls_get(owner, name, pr_model.number())
                .await?;

            // Update status
            StatusLogic::update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                &repo_model,
                &pr_model,
                &upstream_pr,
            )
            .await?;
        }
    }

    Ok(())
}
