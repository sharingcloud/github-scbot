//! Checks logic.

use github_scbot_conf::Config;
use github_scbot_database::models::{HistoryWebhookModel, IDatabaseAdapter, RepositoryModel};
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{
    checks::{GhCheckConclusion, GhCheckSuiteAction, GhCheckSuiteEvent},
    events::EventType,
    status::CheckStatus,
};

use crate::{pulls::PullRequestLogic, status::StatusLogic, Result};

/// Handle GitHub check syite event.
pub async fn handle_check_suite_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhCheckSuiteEvent,
) -> Result<()> {
    let repo_model = RepositoryModel::create_or_update_from_github(
        config.clone(),
        db_adapter.repository(),
        &event.repository,
    )
    .await?;

    // Only look for first PR
    if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
        if let Ok(mut pr_model) = db_adapter
            .pull_request()
            .get_from_repository_and_number(&repo_model, gh_pr.number)
            .await
        {
            // Skip non Github Actions checks
            if event.check_suite.app.slug != "github-actions" {
                return Ok(());
            }

            // Skip non up-to-date checks
            if event.check_suite.head_sha != gh_pr.head.sha {
                return Ok(());
            }

            if config.server_enable_history_tracking {
                HistoryWebhookModel::builder(&repo_model, &pr_model)
                    .username(&event.sender.login)
                    .event_key(EventType::CheckSuite)
                    .payload(&event)
                    .create(db_adapter.history_webhook())
                    .await?;
            }

            // Skip if checks are skipped
            if pr_model.check_status() == CheckStatus::Skipped {
                return Ok(());
            }

            if let GhCheckSuiteAction::Completed = event.action {
                match event.check_suite.conclusion {
                    Some(GhCheckConclusion::Success) => {
                        // Check if other checks are still running
                        let status = PullRequestLogic::get_checks_status_from_github(
                            api_adapter,
                            &repo_model.owner,
                            &repo_model.name,
                            &gh_pr.head.sha,
                            &[event.check_suite.id],
                        )
                        .await?;

                        // Update check status
                        let update = pr_model.create_update().check_status(status).build_update();

                        db_adapter
                            .pull_request()
                            .update(&mut pr_model, update)
                            .await?;
                    }
                    Some(GhCheckConclusion::Failure) => {
                        // Update check status
                        let update = pr_model
                            .create_update()
                            .check_status(CheckStatus::Fail)
                            .build_update();

                        db_adapter
                            .pull_request()
                            .update(&mut pr_model, update)
                            .await?;
                    }
                    _ => (),
                }
            }

            // Update status
            StatusLogic::update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                &repo_model,
                &mut pr_model,
                &event.check_suite.head_sha,
            )
            .await?;
        }
    }

    Ok(())
}
