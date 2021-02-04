//! Checks logic.

use github_scbot_database::{models::PullRequestModel, DbConn};
use github_scbot_types::{
    checks::{GHCheckConclusion, GHCheckSuiteAction, GHCheckSuiteEvent},
    status::CheckStatus,
};

use crate::{
    database::{apply_pull_request_step, process_repository},
    status::{post_status_comment, update_pull_request_status},
    Result,
};

/// Handle GitHub check syite event.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `event` - GitHub check suite event
pub async fn handle_check_suite_event(conn: &DbConn, event: &GHCheckSuiteEvent) -> Result<()> {
    let repo_model = process_repository(conn, &event.repository)?;

    // Only look for first PR
    if let Some(pr_number) = event.check_suite.pull_requests.get(0).map(|x| x.number) {
        let pr_model = PullRequestModel::get_from_repository_id_and_number(
            conn,
            repo_model.id,
            pr_number as i32,
        );

        if let Some(mut pr_model) = pr_model {
            if let GHCheckSuiteAction::Completed = event.action {
                match event.check_suite.conclusion {
                    Some(GHCheckConclusion::Success) => {
                        // Update check status
                        pr_model.set_checks_status(CheckStatus::Pass);
                        pr_model.set_step_auto();
                        pr_model.save(conn)?;
                    }
                    Some(GHCheckConclusion::Failure) => {
                        // Update check status
                        pr_model.set_checks_status(CheckStatus::Fail);
                        pr_model.set_step_auto();
                        pr_model.save(conn)?;
                    }
                    _ => (),
                }
            }

            // Update status message
            apply_pull_request_step(&repo_model, &pr_model).await?;
            post_status_comment(conn, &repo_model, &mut pr_model).await?;
            update_pull_request_status(
                conn,
                &repo_model,
                &mut pr_model,
                &event.check_suite.head_sha,
            )
            .await?;
        }
    }

    Ok(())
}
