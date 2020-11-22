//! Webhook handlers

use actix_web::{web, Error, HttpRequest, HttpResponse};
use log::error;

use super::constants::GITHUB_EVENT_HEADER;
use super::logic::{process_pull_request, process_repository};
use super::types::{
    CheckRunEvent, CheckSuiteEvent, EventType, IssueCommentEvent, PingEvent, PullRequestAction,
    PullRequestEvent, PullRequestReviewCommentEvent, PullRequestReviewEvent, PushEvent,
};
use super::utils::convert_payload_to_string;
use crate::api::comments::post_welcome_comment;
use crate::database::{models::DbConn, DbPool};

pub async fn ping_event(conn: &DbConn, event: PingEvent) -> Result<HttpResponse, Error> {
    process_repository(conn, &event.repository).map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Ping."))
}

pub async fn push_event(conn: &DbConn, event: PushEvent) -> Result<HttpResponse, Error> {
    process_repository(conn, &event.repository).map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Push."))
}

pub async fn pull_request_event(
    conn: &DbConn,
    event: PullRequestEvent,
) -> Result<HttpResponse, Error> {
    process_pull_request(conn, &event.repository, &event.pull_request)
        .map_err(|_| HttpResponse::InternalServerError())?;

    if let PullRequestAction::Opened = event.action {
        post_welcome_comment(
            &event.repository.owner.login,
            &event.repository.name,
            event.pull_request.number,
            &event.pull_request.user.login,
        )
        .await
        .map_err(|_| HttpResponse::InternalServerError())?;
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}

pub async fn pull_request_review_event(
    conn: &DbConn,
    event: PullRequestReviewEvent,
) -> Result<HttpResponse, Error> {
    process_pull_request(conn, &event.repository, &event.pull_request)
        .map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Pull request review."))
}

pub async fn pull_request_review_comment_event(
    conn: &DbConn,
    event: PullRequestReviewCommentEvent,
) -> Result<HttpResponse, Error> {
    process_pull_request(conn, &event.repository, &event.pull_request)
        .map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Pull request review comment."))
}

pub async fn issue_comment_event(
    conn: &DbConn,
    event: IssueCommentEvent,
) -> Result<HttpResponse, Error> {
    process_repository(conn, &event.repository).map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Issue comment."))
}

pub async fn check_run_event(conn: &DbConn, event: CheckRunEvent) -> Result<HttpResponse, Error> {
    process_repository(conn, &event.repository).map_err(|_| HttpResponse::InternalServerError())?;

    Ok(HttpResponse::Ok().body("Check run."))
}

pub async fn check_suite_event(
    conn: &DbConn,
    event: CheckSuiteEvent,
) -> Result<HttpResponse, Error> {
    process_repository(conn, &event.repository).map_err(|_| HttpResponse::InternalServerError())?;

    println!("{:#?}", event);

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
                e => Ok(HttpResponse::Ok().body(format!(
                    "Event handling is not yet implemented for {}",
                    e.as_str()
                ))),
            }
        } else {
            Ok(HttpResponse::BadRequest()
                .body(format!("Bad payload for event '{}'.", event_type.as_str())))
        }
    } else {
        Ok(HttpResponse::Ok().body("Unhandled event."))
    }
}
