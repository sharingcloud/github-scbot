//! Check webhook handlers.

use std::sync::Arc;

use actix_web::HttpResponse;
use prbot_core::use_cases::checks::HandleCheckSuiteEventInterface;
use prbot_ghapi_interface::types::GhCheckSuiteEvent;
use shaku::HasComponent;

use super::parse_event_type;
use crate::{event_type::EventType, server::AppContext, Result, ServerError};

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    ctx: Arc<AppContext>,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    tokio::spawn(async move {
        let ctx = ctx.as_core_context();
        let handle_check_suite_event: &dyn HandleCheckSuiteEventInterface =
            ctx.core_module.resolve_ref();
        handle_check_suite_event
            .run(&ctx, event)
            .await
            .map_err(|e| ServerError::DomainError { source: e })
            .unwrap()
    });

    Ok(HttpResponse::Accepted().body("Check suite."))
}
