//! Issue webhook handlers.

use std::sync::Arc;

use actix_web::HttpResponse;
use prbot_core::use_cases::comments::HandleIssueCommentEventInterface;
use prbot_ghapi_interface::types::GhIssueCommentEvent;
use shaku::HasComponent;

use super::parse_event_type;
use crate::{event_type::EventType, server::AppContext, Result};

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    ctx: Arc<AppContext>,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    tokio::spawn(async move {
        let ctx = ctx.as_core_context();
        let handle_issue_comment_event: &dyn HandleIssueCommentEventInterface =
            ctx.core_module.resolve_ref();
        handle_issue_comment_event.run(&ctx, event).await.unwrap();
    });

    Ok(HttpResponse::Accepted().body("Issue comment."))
}
