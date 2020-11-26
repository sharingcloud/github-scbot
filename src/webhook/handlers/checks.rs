//! Webhook check handlers

use std::convert::TryInto;

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::webhook::logic::{apply_pull_request_step, post_status_comment, process_repository};
use crate::webhook::types::{CheckConclusion, CheckRunEvent, CheckSuiteAction, CheckSuiteEvent};
use crate::{
    api::labels::StepLabel,
    database::models::{CheckStatus, DbConn, PullRequestModel},
};

pub async fn check_run_event(conn: &DbConn, event: CheckRunEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!("Check run event from repository '{}', name '{}', action '{:?}', status '{:?}', conclusion '{:?}'", event.repository.full_name, event.check_run.name, event.action, event.check_run.status, event.check_run.conclusion);

    Ok(HttpResponse::Ok().body("Check run."))
}

pub async fn check_suite_event(conn: &DbConn, event: CheckSuiteEvent) -> Result<HttpResponse> {
    let repo_model = process_repository(conn, &event.repository)?;

    // Only look for first PR
    if let Some(pr_number) = event.check_suite.pull_requests.get(0).map(|x| x.number) {
        let pr_model =
            PullRequestModel::get_from_number(conn, repo_model.id, pr_number.try_into()?);
        if let Some(mut pr_model) = pr_model {
            if let CheckSuiteAction::Completed = event.action {
                match event.check_suite.conclusion {
                    Some(CheckConclusion::Success) => {
                        // Update check status
                        pr_model.update_check_status(conn, Some(CheckStatus::Pass))?;
                        pr_model.update_step(conn, Some(StepLabel::AwaitingReview))?;
                    }
                    Some(CheckConclusion::Failure) => {
                        // Update check status
                        pr_model.update_check_status(conn, Some(CheckStatus::Fail))?;
                        pr_model.update_step(conn, Some(StepLabel::AwaitingChecksChanges))?;
                    }
                    _ => (),
                }
            }

            // Update status message
            let comment_id = post_status_comment(&repo_model, &pr_model).await?;
            apply_pull_request_step(&repo_model, &pr_model).await?;

            pr_model.update_status_comment(conn, comment_id)?;
        }
    }

    info!(
        "Check suite event from repository '{}', action '{:?}', status '{:?}', conclusion '{:?}'",
        event.repository.full_name,
        event.action,
        event.check_suite.status,
        event.check_suite.conclusion
    );

    Ok(HttpResponse::Ok().body("Check suite."))
}
