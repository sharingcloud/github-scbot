//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::IDatabaseAdapter;
use github_scbot_logic::pulls::{handle_pull_request_event, handle_pull_request_opened};
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{
    events::EventType,
    pulls::{GhPullRequestAction, GhPullRequestEvent},
};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_pull_request_event(body: &str) -> Result<GhPullRequestEvent> {
    parse_event_type(EventType::PullRequest, body)
}

pub(crate) async fn pull_request_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    info!(
        repository_path = %event.repository.full_name,
        pull_request_number = event.pull_request.number,
        action = ?event.action,
        author = %event.pull_request.user.login,
        message = "Pull request event",
    );

    if matches!(event.action, GhPullRequestAction::Opened) {
        handle_pull_request_opened(config, api_adapter, db_adapter, redis_adapter, event).await?;
    } else {
        handle_pull_request_event(config, api_adapter, db_adapter, redis_adapter, event).await?;
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}
