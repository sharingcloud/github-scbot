//! Webhook handler tests

use actix_web::{
    dev::MessageBody,
    http, test,
    web::{self, Bytes, BytesMut},
    HttpResponse,
};
use futures::StreamExt;
use github_scbot_api::constants::ENV_API_DISABLE_CLIENT;
use github_scbot_database::establish_test_connection;
use github_scbot_types::events::EventType;

use super::fixtures;
use crate::handlers::event_handler;

fn test_init() {
    dotenv::dotenv().unwrap();
    std::env::set_var(ENV_API_DISABLE_CLIENT, "1");
}

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
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::Ping.to_str())
        .set_payload(fixtures::PUSH_EVENT_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Ping.");
}

#[actix_rt::test]
async fn test_check_suite_completed() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckSuite.to_str())
        .set_payload(fixtures::CHECK_SUITE_COMPLETED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check suite.");
}

#[actix_rt::test]
async fn test_check_run_created() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.to_str())
        .set_payload(fixtures::CHECK_RUN_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check run.");
}

#[actix_rt::test]
async fn test_check_run_completed() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.to_str())
        .set_payload(fixtures::CHECK_RUN_COMPLETED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Check run.");
}

#[actix_rt::test]
async fn test_issue_comment_created() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::IssueComment.to_str())
        .set_payload(fixtures::ISSUE_COMMENT_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Issue comment.");
}

#[actix_rt::test]
async fn test_pull_request_opened() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.to_str())
        .set_payload(fixtures::PULL_REQUEST_OPENED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();

    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_labeled() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.to_str())
        .set_payload(fixtures::PULL_REQUEST_LABELED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_review_comment_created() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header(
            "X-GitHub-Event",
            EventType::PullRequestReviewComment.to_str(),
        )
        .set_payload(fixtures::PULL_REQUEST_REVIEW_COMMENT_CREATED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request review comment.");
}

#[actix_rt::test]
async fn test_pull_request_review_submitted() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequestReview.to_str())
        .set_payload(fixtures::PULL_REQUEST_REVIEW_SUBMITTED_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Pull request review.");
}

#[actix_rt::test]
async fn test_push() {
    test_init();

    let pool = establish_test_connection().unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::Push.to_str())
        .set_payload(fixtures::PUSH_DATA)
        .to_http_parts();

    let resp = event_handler(req, web::Payload(payload), web::Data::new(pool))
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let data = read_body(resp).await;
    assert_eq!(data.to_vec(), b"Push.");
}
