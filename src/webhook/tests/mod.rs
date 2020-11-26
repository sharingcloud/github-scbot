//! Webhook tests

mod fixtures;

use actix_web::test;
use actix_web::{
    dev::MessageBody,
    http,
    web::{self, Bytes, BytesMut},
    HttpResponse,
};
use futures::StreamExt;

use super::handlers::event_handler;
use super::types::EventType;
use crate::database::establish_connection;

async fn read_body<B>(mut res: HttpResponse<B>) -> Bytes
where
    B: MessageBody + Unpin,
{
    let mut body = res.take_body();
    let mut bytes = BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    bytes.freeze()
}

#[actix_rt::test]
async fn test_ping_event() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::Ping.as_str())
        .set_payload(fixtures::PUSH_EVENT_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Ping.");
}

#[actix_rt::test]
async fn test_check_suite_completed() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckSuite.as_str())
        .set_payload(fixtures::CHECK_SUITE_COMPLETED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check suite.");
}

#[actix_rt::test]
async fn test_check_run_created() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.as_str())
        .set_payload(fixtures::CHECK_RUN_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check run.");
}

#[actix_rt::test]
async fn test_check_run_completed() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.as_str())
        .set_payload(fixtures::CHECK_RUN_COMPLETED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check run.");
}

#[actix_rt::test]
async fn test_issue_comment_created() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::IssueComment.as_str())
        .set_payload(fixtures::ISSUE_COMMENT_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Issue comment.");
}

#[actix_rt::test]
async fn test_pull_request_opened() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.as_str())
        .set_payload(fixtures::PULL_REQUEST_OPENED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");

    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_labeled() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.as_str())
        .set_payload(fixtures::PULL_REQUEST_LABELED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_review_comment_created() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header(
            "X-GitHub-Event",
            EventType::PullRequestReviewComment.as_str(),
        )
        .set_payload(fixtures::PULL_REQUEST_REVIEW_COMMENT_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request review comment.");
}

#[actix_rt::test]
async fn test_pull_request_review_submitted() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequestReview.as_str())
        .set_payload(fixtures::PULL_REQUEST_REVIEW_SUBMITTED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request review.");
}

#[actix_rt::test]
async fn test_push() {
    let pool = establish_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::Push.as_str())
        .set_payload(fixtures::PUSH_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .expect("Call should work");
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Push.");
}
