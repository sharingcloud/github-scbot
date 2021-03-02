//! Webhook handler tests

use actix_web::{
    dev::MessageBody,
    http, test,
    web::{self, Bytes, BytesMut},
    HttpResponse,
};
use futures::StreamExt;
use github_scbot_conf::Config;
use github_scbot_database::{models::HistoryWebhookModel, tests::using_test_db, Result};
use github_scbot_types::events::EventType;

use super::fixtures;
use crate::{server::AppContext, webhook::event_handler, ServerError};

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
    ($evt: expr, $payload_json: expr, $config: expr, $pool: expr, $res: expr) => {
        let (req, payload) = test::TestRequest::default()
            .header("Content-Type", "application/json")
            .header("X-GitHub-Event", $evt.to_str())
            .set_payload($payload_json)
            .to_http_parts();

        let resp = event_handler(
            req,
            web::Payload(payload),
            web::Data::new(AppContext {
                config: $config.clone(),
                pool: $pool.clone(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);

        let data = read_body(resp).await;
        assert_eq!(data.to_vec(), $res);
    };
}

#[actix_rt::test]
async fn test_events() -> Result<()> {
    let config = test_config();

    using_test_db(&config.clone(), "test_webhook_ping", |pool| async move {
        test_event!(
            EventType::Ping,
            fixtures::PUSH_EVENT_DATA,
            config,
            pool,
            b"Ping."
        );
        test_event!(
            EventType::CheckSuite,
            fixtures::CHECK_SUITE_COMPLETED_DATA,
            config,
            pool,
            b"Check suite."
        );
        test_event!(
            EventType::CheckRun,
            fixtures::CHECK_RUN_CREATED_DATA,
            config,
            pool,
            b"Check run."
        );
        test_event!(
            EventType::CheckRun,
            fixtures::CHECK_RUN_COMPLETED_DATA,
            config,
            pool,
            b"Check run."
        );
        test_event!(
            EventType::IssueComment,
            fixtures::ISSUE_COMMENT_CREATED_DATA,
            config,
            pool,
            b"Issue comment."
        );
        test_event!(
            EventType::PullRequest,
            fixtures::PULL_REQUEST_OPENED_DATA,
            config,
            pool,
            b"Pull request."
        );

        // Get history
        {
            let conn = pool.get().unwrap();
            let histories = HistoryWebhookModel::list(&conn).unwrap();
            assert_eq!(histories.len(), 1);
            assert_eq!(histories[0].id, 1);
            assert_eq!(histories[0].event_key, EventType::PullRequest.to_str());
        }

        test_event!(
            EventType::PullRequest,
            fixtures::PULL_REQUEST_LABELED_DATA,
            config,
            pool,
            b"Pull request."
        );
        test_event!(
            EventType::PullRequestReview,
            fixtures::PULL_REQUEST_REVIEW_SUBMITTED_DATA,
            config,
            pool,
            b"Pull request review."
        );

        Ok::<_, ServerError>(())
    })
    .await
}
