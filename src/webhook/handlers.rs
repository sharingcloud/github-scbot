//! Webhook handlers

use actix_web::{web::Payload, Error, HttpRequest, HttpResponse, Responder};

use super::constants::GITHUB_EVENT_HEADER;
use super::types::{
    CheckRunEvent, CheckSuiteEvent, EventType, IssueCommentEvent, PingEvent, PullRequestEvent,
    PullRequestReviewCommentEvent, PullRequestReviewEvent, PushEvent,
};
use super::utils::convert_payload_to_string;

pub async fn ping_event(event: PingEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Ping.")
}

pub async fn push_event(event: PushEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Push.")
}

pub async fn pull_request_event(event: PullRequestEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Pull request.")
}

pub async fn pull_request_review_event(event: PullRequestReviewEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Pull request review.")
}

pub async fn pull_request_review_comment_event(
    event: PullRequestReviewCommentEvent,
) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Pull request review comment.")
}

pub async fn issue_comment_event(event: IssueCommentEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Issue comment.")
}

pub async fn check_run_event(event: CheckRunEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Check run.")
}

pub async fn check_suite_event(event: CheckSuiteEvent) -> HttpResponse {
    println!("{:#?}", event);

    HttpResponse::Ok().body("Check suite.")
}

pub async fn event_handler(req: HttpRequest, mut payload: Payload) -> Result<HttpResponse, Error> {
    // Route event depending on header
    if let Ok(Some(event_type)) = req
        .headers()
        .get(GITHUB_EVENT_HEADER)
        .map(|x| EventType::try_from_str(x.to_str()?))
        .map_or(Ok(None), |r| r.map(Some))
    {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            match event_type {
                EventType::CheckRun => Ok(check_run_event(serde_json::from_str(&body)?).await),
                EventType::CheckSuite => Ok(check_suite_event(serde_json::from_str(&body)?).await),
                EventType::IssueComment => {
                    Ok(issue_comment_event(serde_json::from_str(&body)?).await)
                }
                EventType::Ping => Ok(ping_event(serde_json::from_str(&body)?).await),
                EventType::PullRequest => {
                    Ok(pull_request_event(serde_json::from_str(&body)?).await)
                }
                EventType::PullRequestReview => {
                    Ok(pull_request_review_event(serde_json::from_str(&body)?).await)
                }
                EventType::PullRequestReviewComment => {
                    Ok(pull_request_review_comment_event(serde_json::from_str(&body)?).await)
                }
                EventType::Push => Ok(push_event(serde_json::from_str(&body)?).await),
                e => Ok(HttpResponse::Ok().body(format!(
                    "Event handling in to be implemented for {}",
                    e.as_str()
                ))),
            }
        } else {
            Ok(HttpResponse::BadRequest().body("Bad request body."))
        }
    } else {
        Ok(HttpResponse::Ok().body("Unhandled event."))
    }
}

pub async fn get_handler(_req: HttpRequest) -> impl Responder {
    println!("In get_handler");

    HttpResponse::Ok().body("Get.")
}
