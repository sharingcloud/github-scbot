//! Checks logic.

use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{HistoryWebhookModel, PullRequestModel, RepositoryModel},
    DbPool,
};
use github_scbot_types::{
    checks::{GhCheckConclusion, GhCheckSuiteAction, GhCheckSuiteEvent},
    events::EventType,
    status::CheckStatus,
};

use crate::{pulls::get_checks_status_from_github, status::update_pull_request_status, Result};

/// Handle GitHub check syite event.
pub async fn handle_check_suite_event(
    config: Config,
    pool: DbPool,
    event: GhCheckSuiteEvent,
) -> Result<()> {
    let repo_model = RepositoryModel::create_or_update_from_github(
        config.clone(),
        pool.clone(),
        event.repository.clone(),
    )
    .await?;

    // Only look for first PR
    if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
        let conn = get_connection(&pool)?;
        if let Ok(mut pr_model) =
            PullRequestModel::get_from_repository_and_number(&conn, &repo_model, gh_pr.number)
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
                    .create(&conn)?;
            }

            if let GhCheckSuiteAction::Completed = event.action {
                match event.check_suite.conclusion {
                    Some(GhCheckConclusion::Success) => {
                        // Check if other checks are still running
                        let status = get_checks_status_from_github(
                            &config,
                            &repo_model.owner,
                            &repo_model.name,
                            &gh_pr.head.sha,
                            &[event.check_suite.id],
                        )
                        .await?;

                        // Update check status
                        pr_model.set_checks_status(status);
                        pr_model.save(&conn)?;
                    }
                    Some(GhCheckConclusion::Failure) => {
                        // Update check status
                        pr_model.set_checks_status(CheckStatus::Fail);
                        pr_model.save(&conn)?;
                    }
                    _ => (),
                }
            }

            // Update status
            update_pull_request_status(
                &config,
                pool,
                &repo_model,
                &mut pr_model,
                &event.check_suite.head_sha,
            )
            .await?;
        }
    }

    Ok(())
}
