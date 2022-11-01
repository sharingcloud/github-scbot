//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::config::Config;
use github_scbot_core::types::{
    events::EventType,
    pulls::{GhPullRequestAction, GhPullRequestEvent},
};
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_domain::pulls::{handle_pull_request_event, handle_pull_request_opened};
use github_scbot_redis::RedisService;
use snafu::ResultExt;

use super::parse_event_type;
use crate::errors::DomainSnafu;
use crate::errors::Result;

pub(crate) fn parse_pull_request_event(body: &str) -> Result<GhPullRequestEvent> {
    parse_event_type(EventType::PullRequest, body)
}

pub(crate) async fn pull_request_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    if matches!(event.action, GhPullRequestAction::Opened) {
        handle_pull_request_opened(config, api_adapter, db_adapter, redis_adapter, event)
            .await
            .context(DomainSnafu)?;
    } else {
        handle_pull_request_event(api_adapter, db_adapter, redis_adapter, event)
            .await
            .context(DomainSnafu)?;
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}
