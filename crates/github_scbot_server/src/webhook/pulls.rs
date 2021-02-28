//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::{get_connection, DbPool};
use github_scbot_logic::pulls::handle_pull_request_event;
use github_scbot_types::pulls::GHPullRequestEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn pull_request_event(
    config: Config,
    pool: DbPool,
    event: GHPullRequestEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request event from repository '{}', PR number #{}, action '{:?}' (from '{}')",
        event.repository.full_name,
        event.pull_request.number,
        event.action,
        event.pull_request.user.login
    );

    handle_pull_request_event(&config, &*get_connection(&pool)?, &event).await?;
    Ok(HttpResponse::Ok().body("Pull request."))
}
