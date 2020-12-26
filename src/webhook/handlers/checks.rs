//! Webhook check handlers

use std::convert::TryInto;

use actix_web::HttpResponse;
use tracing::info;

use crate::database::models::{CheckStatus, DbConn, PullRequestModel};
use crate::errors::Result;
use crate::webhook::logic::{
    database::{apply_pull_request_step, process_repository},
    status::post_status_comment,
};
use crate::webhook::types::{CheckConclusion, CheckRunEvent, CheckSuiteAction, CheckSuiteEvent};

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
                        pr_model.update_step_auto(conn)?;
                    }
                    Some(CheckConclusion::Failure) => {
                        // Update check status
                        pr_model.update_check_status(conn, Some(CheckStatus::Fail))?;
                        pr_model.update_step_auto(conn)?;
                    }
                    _ => (),
                }
            }

            // Update status message
            post_status_comment(conn, &repo_model, &mut pr_model).await?;
            apply_pull_request_step(&repo_model, &pr_model).await?;
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
