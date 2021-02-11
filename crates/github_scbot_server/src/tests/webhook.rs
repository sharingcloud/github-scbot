//! Webhook handler tests

use actix_web::{
    dev::MessageBody,
    http, test,
    web::{self, Bytes, BytesMut},
    HttpResponse,
};
use futures::StreamExt;
use github_scbot_core::Config;
use github_scbot_database::establish_test_connection;
use github_scbot_types::events::EventType;

use super::fixtures;
use crate::{server::AppContext, webhook::event_handler};

fn test_config() -> Config {
    let mut config = Config::from_env();
    config.api_disable_client = true;
    config
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

macro_rules! test_event {
    ($req: tt, $payload: tt, $config: tt, $pool: tt, $res: tt) => {
        let resp = event_handler(
            $req,
            web::Payload($payload),
            web::Data::new(AppContext { $config, $pool }),
        )
        .await
        .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);

        let data = read_body(resp).await;
        assert_eq!(data.to_vec(), $res);
    };
}

#[actix_rt::test]
async fn test_ping_event() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::Ping.to_str())
        .set_payload(fixtures::PUSH_EVENT_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Ping.");
}

#[actix_rt::test]
async fn test_check_suite_completed() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckSuite.to_str())
        .set_payload(fixtures::CHECK_SUITE_COMPLETED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Check suite.");
}

#[actix_rt::test]
async fn test_check_run_created() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.to_str())
        .set_payload(fixtures::CHECK_RUN_CREATED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Check run.");
}

#[actix_rt::test]
async fn test_check_run_completed() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::CheckRun.to_str())
        .set_payload(fixtures::CHECK_RUN_COMPLETED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Check run.");
}

#[actix_rt::test]
async fn test_issue_comment_created() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::IssueComment.to_str())
        .set_payload(fixtures::ISSUE_COMMENT_CREATED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Issue comment.");
}

#[actix_rt::test]
async fn test_pull_request_opened() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.to_str())
        .set_payload(fixtures::PULL_REQUEST_OPENED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_labeled() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequest.to_str())
        .set_payload(fixtures::PULL_REQUEST_LABELED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Pull request.");
}

#[actix_rt::test]
async fn test_pull_request_review_comment_created() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header(
            "X-GitHub-Event",
            EventType::PullRequestReviewComment.to_str(),
        )
        .set_payload(fixtures::PULL_REQUEST_REVIEW_COMMENT_CREATED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Pull request review comment.");
}

#[actix_rt::test]
async fn test_pull_request_review_submitted() {
    let config = test_config();

    let pool = establish_test_connection(&config).unwrap();
    let (req, payload) = test::TestRequest::default()
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", EventType::PullRequestReview.to_str())
        .set_payload(fixtures::PULL_REQUEST_REVIEW_SUBMITTED_DATA)
        .to_http_parts();

    test_event!(req, payload, config, pool, b"Pull request review.");
}
