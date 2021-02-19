//! Review webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbConn;
use github_scbot_logic::reviews::handle_review_event;
use github_scbot_types::reviews::GHReviewEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn review_event(
    config: &Config,
    conn: &DbConn,
    event: GHReviewEvent,
) -> Result<HttpResponse> {
    info!(
        "Pull request review event from repository '{}', PR number #{}, action '{:?}' (review from '{}')",
        event.repository.full_name, event.pull_request.number, event.action, event.review.user.login
    );

    handle_review_event(config, conn, &event).await?;
    Ok(HttpResponse::Ok().body("Pull request review."))
}
