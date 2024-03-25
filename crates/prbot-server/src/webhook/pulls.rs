//! Pull webhook handlers.

use std::sync::Arc;

use actix_web::HttpResponse;
use prbot_core::use_cases::pulls::ProcessPullRequestEventInterface;
use prbot_ghapi_interface::types::GhPullRequestEvent;
use shaku::HasComponent;

use super::parse_event_type;
use crate::{event_type::EventType, server::AppContext, Result, ServerError};

pub(crate) fn parse_pull_request_event(body: &str) -> Result<GhPullRequestEvent> {
    parse_event_type(EventType::PullRequest, body)
}

#[tracing::instrument(skip_all, fields(
    action = ?event.action,
    repo_owner = event.repository.owner.login,
    repo_name = event.repository.name,
    pr_number = event.pull_request.number,
))]
pub(crate) async fn pull_request_event(
    ctx: Arc<AppContext>,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    tokio::spawn(async move {
        let ctx = ctx.as_core_context();

        let handle_pull_request_event: &dyn ProcessPullRequestEventInterface =
            ctx.core_module.resolve_ref();
        handle_pull_request_event
            .run(&ctx, event)
            .await
            .map_err(|e| ServerError::DomainError { source: e })
            .unwrap();
    });

    Ok(HttpResponse::Accepted().body("Pull request."))
}
