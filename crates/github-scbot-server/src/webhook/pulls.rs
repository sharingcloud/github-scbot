//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::{
    checks::DetermineChecksStatusUseCase,
    comments::PostWelcomeCommentUseCase,
    pulls::{
        AutomergePullRequestUseCase, HandlePullRequestEventUseCase, MergePullRequestUseCase,
        ProcessPullRequestOpenedUseCase, ProcessPullRequestOpenedUseCaseInterface,
        SetStepLabelUseCase,
    },
    status::{BuildPullRequestStatusUseCase, UpdatePullRequestStatusUseCase},
    summary::UpdatePullRequestSummaryUseCase,
};
use github_scbot_ghapi_interface::{
    types::{GhPullRequestAction, GhPullRequestEvent},
    ApiService,
};
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{event_type::EventType, Result, ServerError};

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
    config: &Config,
    api_service: &dyn ApiService,
    db_service: &dyn DbService,
    lock_service: &dyn LockService,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    if matches!(event.action, GhPullRequestAction::Opened) {
        ProcessPullRequestOpenedUseCase {
            api_service,
            db_service,
            config,
            lock_service,
            post_welcome_comment: &PostWelcomeCommentUseCase { api_service },
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
                update_pull_request_summary: &UpdatePullRequestSummaryUseCase { api_service },
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
    } else {
        HandlePullRequestEventUseCase {
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
                update_pull_request_summary: &UpdatePullRequestSummaryUseCase { api_service },
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
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}
