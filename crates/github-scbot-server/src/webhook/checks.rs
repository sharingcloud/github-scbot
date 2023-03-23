//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::{
    checks::{DetermineChecksStatusUseCase, HandleCheckSuiteEventUseCase},
    pulls::{AutomergePullRequestUseCase, MergePullRequestUseCase, SetStepLabelUseCase},
    status::{BuildPullRequestStatusUseCase, UpdatePullRequestStatusUseCase},
    summary::PostSummaryCommentUseCase,
};
use github_scbot_ghapi_interface::{types::GhCheckSuiteEvent, ApiService};
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{event_type::EventType, Result, ServerError};

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    api_service: &dyn ApiService,
    db_service: &dyn DbService,
    lock_service: &dyn LockService,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    HandleCheckSuiteEventUseCase {
        api_service,
        db_service,
        update_pull_request_status: &UpdatePullRequestStatusUseCase {
            api_service,
            db_service,
            lock_service,
            set_step_label: &SetStepLabelUseCase { api_service },
            automerge_pull_request: &AutomergePullRequestUseCase {
                db_service,
                api_service,
                merge_pull_request: &MergePullRequestUseCase { api_service },
            },
            post_summary_comment: &PostSummaryCommentUseCase {
                api_service,
                db_service,
                lock_service,
            },
            build_pull_request_status: &BuildPullRequestStatusUseCase {
                api_service,
                db_service,
                determine_checks_status: &DetermineChecksStatusUseCase { api_service },
            },
        },
    }
    .run(event)
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
