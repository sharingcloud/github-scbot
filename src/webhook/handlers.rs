//! Webhook handlers

use std::convert::TryInto;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use diesel::prelude::*;
use eyre::Result;
use log::{error, info};

use super::constants::GITHUB_EVENT_HEADER;
use super::logic::{process_pull_request, process_repository};
use super::types::{
    CheckConclusion, CheckRunEvent, CheckSuiteAction, CheckSuiteEvent, EventType,
    IssueCommentEvent, PingEvent, PullRequestAction, PullRequestEvent,
    PullRequestReviewCommentEvent, PullRequestReviewEvent, PushEvent,
};
use super::utils::convert_payload_to_string;
use crate::api::comments::{create_or_update_status_comment, post_welcome_comment};
use crate::database::models::{CheckStatus, PullRequestModel};
use crate::database::{models::DbConn, DbPool};

pub async fn ping_event(conn: &DbConn, event: PingEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Ping event from repository '{}'",
        event.repository.full_name
    );

    Ok(HttpResponse::Ok().body("Ping."))
}

pub async fn push_event(conn: &DbConn, event: PushEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Push event from repository '{}', reference '{}' (from '{}')",
        event.repository.full_name, event.reference, event.pusher.name
    );

    Ok(HttpResponse::Ok().body("Push."))
}

pub async fn pull_request_event(conn: &DbConn, event: PullRequestEvent) -> Result<HttpResponse> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    if let PullRequestAction::Opened = event.action {
        post_welcome_comment(
            &event.repository.owner.login,
            &event.repository.name,
            event.pull_request.number,
            &event.pull_request.user.login,
        )
        .await?;
    }

    if let PullRequestAction::Synchronize = event.action {
        // Reset status check
        pr_model.check_status = CheckStatus::Waiting.as_str().to_string();
        pr_model.save_changes::<PullRequestModel>(conn)?;
    }

    let comment_id = create_or_update_status_comment(&repo_model, &pr_model).await?;

    let pr_status_comment_id: u64 = pr_model.status_comment_id.try_into()?;
    if comment_id != pr_status_comment_id {
        pr_model.status_comment_id = pr_status_comment_id.try_into()?;
        pr_model.save_changes::<PullRequestModel>(conn)?;
    }

    info!(
        "Pull request event from repository '{}', PR number #{}, action '{:?}' (from '{}')",
        event.repository.full_name,
        event.pull_request.number,
        event.action,
        event.pull_request.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request."))
}

pub async fn pull_request_review_event(
    conn: &DbConn,
    event: PullRequestReviewEvent,
) -> Result<HttpResponse> {
    process_pull_request(conn, &event.repository, &event.pull_request)?;

    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub async fn pull_request_review_comment_event(
    conn: &DbConn,
    event: PullRequestReviewCommentEvent,
) -> Result<HttpResponse> {
    process_pull_request(conn, &event.repository, &event.pull_request)?;

    info!(
        "Pull request review comment event from repository '{}', PR number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.comment.user.login
    );

    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

pub async fn issue_comment_event(conn: &DbConn, event: IssueCommentEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Issue comment event from repository '{}', issue number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.issue.number, event.action, event.comment.user.login
    );

    Ok(HttpResponse::Ok().body("Issue comment."))
}

pub async fn check_run_event(conn: &DbConn, event: CheckRunEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!("Check run event from repository '{}', name '{}', action '{:?}', status '{:?}', conclusion '{:?}'", event.repository.full_name, event.check_run.name, event.action, event.check_run.status, event.check_run.conclusion);

    Ok(HttpResponse::Ok().body("Check run."))
}

pub async fn check_suite_event(conn: &DbConn, event: CheckSuiteEvent) -> Result<HttpResponse> {
    let repo_model = process_repository(conn, &event.repository)?;

    if let Some(pr_number) = event.check_suite.pull_requests.get(0).map(|x| x.number) {
        let pr_model =
            PullRequestModel::get_from_number(conn, repo_model.id, pr_number.try_into()?);
        if let Some(mut pr_model) = pr_model {
            if let CheckSuiteAction::Completed = event.action {
                match event.check_suite.conclusion {
                    Some(CheckConclusion::Success) => {
                        // Update check status
                        pr_model.check_status = CheckStatus::Pass.as_str().to_string();
                        let pr_model = pr_model.save_changes::<PullRequestModel>(conn)?;

                        eprintln!(
                            "PR #{} check status changed to {:?}",
                            pr_model.number, pr_model.check_status
                        );
                    }
                    Some(CheckConclusion::Failure) => {
                        // Update check status
                        pr_model.check_status = CheckStatus::Fail.as_str().to_string();
                        let pr_model = pr_model.save_changes::<PullRequestModel>(conn)?;

                        eprintln!(
                            "PR #{} check status changed to {:?}",
                            pr_model.number, pr_model.check_status
                        );
                    }
                    _ => (),
                }
            } else {
                // Requested/re-requested
                pr_model.check_status = CheckStatus::Waiting.as_str().to_string();
                let pr_model = pr_model.save_changes::<PullRequestModel>(conn)?;

                eprintln!(
                    "PR #{} check status changed to {:?}",
                    pr_model.number, pr_model.check_status
                );
            }

            // Update status message
            let comment_id = create_or_update_status_comment(&repo_model, &pr_model).await?;
            let pr_status_comment_id: u64 = pr_model.status_comment_id.try_into()?;
            if comment_id != pr_status_comment_id {
                pr_model.status_comment_id = pr_status_comment_id.try_into()?;
                let pr_model = pr_model.save_changes::<PullRequestModel>(conn)?;

                eprintln!(
                    "PR #{} status comment ID changed to {:?}",
                    pr_model.number, pr_model.status_comment_id
                );
            }
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

pub async fn event_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    // Route event depending on header
    if let Ok(Some(event_type)) = req
        .headers()
        .get(GITHUB_EVENT_HEADER)
        .map(|x| EventType::try_from_str(x.to_str()?))
        .map_or(Ok(None), |r| r.map(Some))
    {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            let conn = pool.get().map_err(|_| {
                error!("Error while getting connection from database pool.");
                HttpResponse::InternalServerError()
            })?;

            match event_type {
                EventType::CheckRun => check_run_event(&conn, serde_json::from_str(&body)?).await,
                EventType::CheckSuite => {
                    check_suite_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::IssueComment => {
                    issue_comment_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::Ping => ping_event(&conn, serde_json::from_str(&body)?).await,
                EventType::PullRequest => {
                    pull_request_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::PullRequestReview => {
                    pull_request_review_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::PullRequestReviewComment => {
                    pull_request_review_comment_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::Push => push_event(&conn, serde_json::from_str(&body)?).await,
                e => Ok(HttpResponse::BadRequest().body(format!(
                    "Event handling is not yet implemented for {}",
                    e.as_str()
                ))),
            }
            .map_err(|e| {
                eprintln!("ERROR: {:?}", e);
                error::ErrorInternalServerError(e)
            })
        } else {
            Ok(HttpResponse::BadRequest()
                .body(format!("Bad payload for event '{}'.", event_type.as_str())))
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Unhandled event."))
    }
}
