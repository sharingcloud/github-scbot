//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::pulls::{handle_pull_request_event, handle_pull_request_opened};
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
    config: Config,
    pool: DbPool,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request event from repository '{}', PR number #{}, action '{:?}' (from '{}')",
        event.repository.full_name,
        event.pull_request.number,
        event.action,
        event.pull_request.user.login
    );

    if matches!(event.action, GhPullRequestAction::Opened) {
        handle_pull_request_opened(config, pool, event).await?;
    } else {
        handle_pull_request_event(config, pool, event).await?;
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}
